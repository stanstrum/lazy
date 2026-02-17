use std::cmp::Ordering;
use crate::error::{Level, MessageContents, MessageSection, PrintableMessage, print_message};
use crate::lang::reference::{ExpressionReference, Store};
use crate::tokenize::token::{GroupingKind, GroupingType, Span};

use super::*;

pub(super) fn make_block<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  module: lang::reference::ModuleReference,
  function: lang::reference::FunctionReference,
) -> Result<Option<lang::reference::BlockReference>, Error> {
  let indenter = stream.indenter_here()?;

  let Some((Token::Grouping(GroupingType::Open(GroupingKind::Brace)), start)) = stream.peek()? else {
    return Ok(None)
  };

  stream.seek();
  stream.skip_whitespace_and_comments()?;

  if let Some((Token::Grouping(GroupingType::Close(GroupingKind::Brace)), end)) = stream.peek()? {
    stream.seek();

    let span = Span::from_pair(start, end);

    let empty_block = lang::expr::BlockExpression::new(
      Span::from_pair(start, end),
      lang::ty::Type::Intrinsic { kind: lang::ty::Intrinsic::Void, span },
    );
    let id = lazy.rget_mut(function).add_block(empty_block);

    return Ok(Some(lang::reference::BlockReference(function, id)));
  };

  let Some((Token::Indent(0..), _)) = indenter.peek(stream)? else {
    return stream.expected_here(line_dbg!("a newline (positive indent)"));
  };
  stream.seek();

  let mut children = vec![];
  let mut variables = vec![];
  let mut non_return_last = None;
  loop {
    stream.skip_whitespace_and_comments()?;

    if let Some((Token::Indent(indent), _)) = indenter.peek(stream)? {
      match indent.cmp(&0) {
        Ordering::Less => {
          stream.seek();
          break;
        },
        Ordering::Equal => {
          stream.seek();
          continue;
        },
        Ordering::Greater => {
          return Err(Error::Invalid {
            what: line_dbg!("newline (positive indent)"),
            at: stream.here()?,
          });
        },
      };
    };

    let expr = {
      if let Some((variable, assignment)) = variable::make_assignment(lazy, stream, module, function)? {
        let function_ref = lazy.rget(function);

        let variable_names = variables.iter().map(|x: &lang::expr::Variable| &x.name);
        let argument_names = function_ref.header.arguments.iter().map(|x| &x.name);

        let conflict = argument_names.chain(variable_names)
          .find(|prior: &&lang::module::Name| prior.id == variable.name.id);

        if let Some(conflict) = conflict {
          print_message(lazy, PrintableMessage {
            level: Level::Warn,
            force: false,
            description: "conflicting name will be shadowed".into(),
            contents: MessageContents::WithinSource {
              range: function_ref.span,
              sections: vec![
                MessageSection {
                  text: "first used here".into(),
                  span: conflict.span,
                },
                MessageSection {
                  text: "shadowed here".into(),
                  span: variable.name.span,
                },
              ],
            },
          });
        };

        variables.push(variable);
        assignment
      } else if let Some(expr) = make_expr(lazy, stream, module, function)? {
        Some(expr)
      } else {
        return stream.expected_here(line_dbg!("an assignment or an expression"));
      }
    };

    let id = if let Some(expr) = expr {
      let id = lazy.rget_mut(function).add_expr(expr);
      children.push(id);
      Some(id)
    } else {
      None
    };

    stream.skip_whitespace_and_comments()?;

    if let Some((Token::Operator(Operator::Semicolon), _)) = indenter.peek(stream)? {
      stream.seek();
      non_return_last = id;

      stream.skip_whitespace_and_comments()?;
    };

    stream.peek()?;
    let Some((Token::Indent(indent), _)) = indenter.peek(stream)? else {
      return stream.expected_here(line_dbg!("a newline"));
    };

    match indent.cmp(&0) {
      Ordering::Less => {
        stream.seek();
        break;
      },
      Ordering::Equal => {
        stream.seek();
      },
      Ordering::Greater => {
        return Err(Error::Invalid {
          what: line_dbg!("newline (positive indent)"),
          at: stream.here()?,
        });
      },
    };
  };

  stream.skip_whitespace_and_comments()?;

  let Some((Token::Grouping(GroupingType::Close(GroupingKind::Brace)), end)) = indenter.peek(stream)? else {
    return stream.expected_here(line_dbg!("a closing brace"));
  };
  stream.seek();

  let span = Span::from_pair(start, end);

  let returns_last = !non_return_last.is_some_and(
    |id| id == *children.last().unwrap()
  );

  let out = if returns_last {
    let function_ref = lazy.rget(function);
    let body = lazy.rget(function_ref.body);
    let &index = body.children.last().unwrap();
    let reference = ExpressionReference(function, index);
    lang::ty::Type::Expression(reference)
  } else {
    lang::ty::Type::Intrinsic {
      kind: lang::ty::Intrinsic::Void,
      span,
    }
  };

  let id = lazy.rget_mut(function).add_block(lang::expr::BlockExpression {
    children,
    span,
    returns_last,
    out,
    variables,
  });

  Ok(Some(lang::reference::BlockReference(function, id)))
}
