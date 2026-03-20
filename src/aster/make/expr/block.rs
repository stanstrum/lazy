use std::cmp::Ordering;
use crate::aster::make::Indenter;
use crate::error::{Level, MessageContents, MessageSection, PrintableMessage, print_message};
use crate::lang::reference::{ExpressionReference, Store};
use crate::lang::span::GetSpan;
use crate::tokenize::token::{GroupingKind, GroupingType, Operator, Span};

use super::*;

pub struct BlockStatement {
  pub end: bool,
  pub non_return_last: bool,
  pub expr: Option<ExpressionReference>,
}

impl BlockStatement {
  fn empty() -> Self {
    Self {
      end: false,
      non_return_last: false,
      expr: None,
    }
  }
}

pub fn make_block_statement<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  indenter: &Indenter,
  module: lang::reference::ModuleReference,
  function: lang::reference::FunctionReference,
  block: lang::reference::BlockReference,
) -> Result<Option<BlockStatement>, Error> {
  if let Some((Token::Indent(indent), _)) = stream.peek()? {
    match indent.cmp(&0) {
      Ordering::Less => {
        stream.seek();
        return Ok(Some(BlockStatement {
          end: true,
          ..BlockStatement::empty()
        }));
      },
      Ordering::Equal => {
        stream.seek();
        return Ok(Some(BlockStatement::empty()));
      },
      Ordering::Greater => {
        return Err(Error::Invalid {
          what: line_dbg!("a newline (0 or negative indent)"),
          at: stream.here()?,
        });
      },
    };
  };

  let expr = if let Some((variable, expr)) = variable::make_assignment(lazy, stream, module, function)? {
    let function_ref = lazy.rget(function);
    let block_ref = lazy.rget(block);

    let variable_names = block_ref.variables.iter().map(|x: &lang::expr::Variable| &x.name);
    let argument_names = function_ref.header.arguments.iter().map(|x| &x.name);

    let conflict = argument_names.chain(variable_names)
      .find(|prior| prior.id == variable.name.id);

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

    let variable_span = variable.span;
    let var_id = block.rget_from(lazy).variables.len();

    lazy.rget_mut(block).variables.push(variable);

    if let Some(b) = expr {
      let span = b.rget_from(lazy).get_span(lazy);

      let variable_reference = lang::reference::VariableReference::Block(block, var_id);

      let a = function.rget_from_mut(lazy).add_expr(lang::expr::Expression::Variable { reference: variable_reference, span: variable_span });
      let a = lang::reference::ExpressionReference(function, a);

      let assignment = lang::expr::Expression::Binary {
        a,
        b,
        op: (lang::expr::operator::BinaryOperator::Assign, variable_span),
        span,
      };

      let id = function.rget_from_mut(lazy).add_expr(assignment);
      Some(lang::reference::ExpressionReference(function, id))
    } else {
      None
    }
  } else if let Some(expr) = make_expr(lazy, stream, module, function)? {
    Some(expr)
  } else {
    return stream.expected_here(line_dbg!("an expression"));
  };

  if let Some(ExpressionReference(here, id)) = expr {
    assert!(function == here);
    block.rget_from_mut(lazy).children.push(id);
  };

  stream.skip_whitespace_and_comments()?;

  let non_return_last = {
    if let Some((Token::Operator(Operator::Semicolon), _)) = stream.peek()? {
      stream.seek();
      stream.skip_whitespace_and_comments()?;

      true
    } else {
      false
    }
  };

  let Some((Token::Indent(indent), _)) = stream.peek()? else {
    return stream.expected_here(line_dbg!("a newline"));
  };

  let end = match indent.cmp(&0) {
    Ordering::Less => {
      stream.seek();
      true
    },
    Ordering::Equal => {
      stream.seek();
      false
    },
    Ordering::Greater => {
      return Err(Error::Invalid {
        what: line_dbg!("a newline (0 or negative indent)"),
        at: stream.here()?,
      });
    },
  };

  Ok(Some(BlockStatement {
    end,
    non_return_last,
    expr,
  }))
}

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

    let empty_block = lang::expr::BlockExpression::new_dirty(span);
    let id = lazy.rget_mut(function).add_block(empty_block);

    return Ok(Some(lang::reference::BlockReference(function, id)));
  };

  let Some((Token::Indent(0..), _)) = indenter.peek(stream)? else {
    return stream.expected_here(line_dbg!("a newline (positive indent)"));
  };
  stream.seek();

  let block = lang::expr::BlockExpression::new_dirty(start);
  let block = lazy.rget_mut(function).add_block(block);
  let block = lang::reference::BlockReference(function, block);

  let mut non_return_last = None;
  loop {
    stream.skip_whitespace_and_comments()?;

    let Some(stmt) = make_block_statement(lazy, stream, &indenter, module, function, block)? else {
      return stream.expected_here(line_dbg!("a block statement"));
    };

    if stmt.non_return_last && stmt.expr.is_some() {
      non_return_last = stmt.expr;
    };

    if stmt.end {
      break;
    };
  };

  stream.skip_whitespace_and_comments()?;

  let Some((Token::Grouping(GroupingType::Close(GroupingKind::Brace)), end)) = indenter.peek(stream)? else {
    return stream.expected_here(line_dbg!("a closing brace"));
  };
  stream.seek();

  let span = Span::from_pair(start, end);

  let children = &lazy.rget(block).children;
  let returns_last = !children.is_empty() && !non_return_last.is_some_and(
    |ExpressionReference(_, id)| id == *children.last().unwrap()
  );

  let out = if returns_last {
    let &index = children.last().unwrap();
    let reference = ExpressionReference(function, index);

    lang::ty::Type::Reference(
      lang::reference::TypeReference::Expression(reference)
    )
  } else {
    lang::ty::Type::Intrinsic {
      kind: lang::ty::Intrinsic::Void,
      span,
    }
  };

  lazy.rget_mut(block).out = out;
  lazy.rget_mut(block).returns_last = returns_last;
  lazy.rget_mut(block).span = span;

  Ok(Some(block))
}
