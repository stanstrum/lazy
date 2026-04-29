use lang::Compiler;
use lazy_macros::print_message;

use std::cmp::Ordering;

use crate::make::Indenter;
use lang::reference::{BlockReference, ExpressionReference, Reference, Store};
use lang::span::GetSpan;
use lang::token::{GroupingKind, GroupingType, Operator};
use lang::span::Span;

use super::*;

pub struct BlockStatement<C: Compiler> {
  pub end: bool,
  pub non_return_last: bool,
  pub expr: Option<ExpressionReference<C>>,
}

impl<C: Compiler> BlockStatement<C> {
  fn empty() -> Self {
    Self {
      end: false,
      non_return_last: false,
      expr: None,
    }
  }
}

pub fn make_block_statement<'pool, C: Compiler, const N: usize, T: Read>(
  store: &mut C::Store<'pool>,
  stream: &mut Rereader<'pool, C, N, T>,
  _indenter: &Indenter,
  module: C::ModuleReference,
  function: C::FunctionReference,
  block: lang::reference::BlockReference<C>,
) -> Result<Option<BlockStatement<C>>, Error<C>> {
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

  let expr = if let Some((variable, expr)) = variable::make_assignment(store, stream, module, block)? {
    let function_ref = <C::Store<'pool> as Store<C::FunctionReference>>::rget(store, function);
    let block_ref = <C::Store<'pool> as Store<BlockReference<C>>>::rget(store, block);

    let variable_names = block_ref.variables.iter().map(|x: &lang::expr::Variable<C>| &x.name);
    let argument_names = function_ref.header.arguments.iter().map(|x| &x.name);

    let conflict = argument_names.chain(variable_names)
      .find(|prior| prior.id == variable.name.id);

    if let Some(conflict) = conflict {
      print_message!(store, {
        level: Warn,
        force: false,
        description: line_dbg!("conflicting name will be shadowed").into(),
        contents: MessageContents::WithinSource(vec![WithinSource {
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
        }]),
      });
    };

    let variable_span = variable.span;
    let var_id = block.rget_from(store).variables.len();

    store.rget_mut(block).variables.push(variable);

    expr.map(|b| {
      let span = b.rget_from(store).get_span(store);

      let variable_reference = lang::reference::VariableReference::Block(block, var_id);

      let a = function.rget_from_mut(store).add_expr(lang::expr::Expression::Variable { reference: variable_reference, span: variable_span });
      let a = ExpressionReference(block, a);

      let assignment = lang::expr::Expression::Binary {
        a,
        b,
        op: (lang::expr::operator::BinaryOperator::Assign, variable_span),
        span,
        out: todo!(),
        // lang::ty::TypeValue::Weak { span },
      };

      let id = function.rget_from_mut(store).add_expr(assignment);
      ExpressionReference(block, id)
    })
  } else if let Some(expr) = make_expr(store, stream, module, block)? {
    Some(expr)
  } else {
    return stream.expected_here(line_dbg!("an expression"));
  };

  if let Some(ExpressionReference(here, id)) = expr {
    assert!(function == here.0);
    block.rget_from_mut(store).children.push(id);
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

pub(super) fn make_block<'pool, C: Compiler, const N: usize, T: Read>(
  store: &mut C::Store<'pool>,
  stream: &mut Rereader<'pool, C, N, T>,
  module: C::ModuleReference,
  function: C::FunctionReference,
  parent: Option<BlockReference<C>>,
) -> Result<Option<BlockReference<C>>, Error<C>> {
  let indenter = stream.indenter_here()?;

  let Some((Token::Grouping(GroupingType::Open(GroupingKind::Brace)), start)) = stream.peek()? else {
    return Ok(None)
  };

  stream.seek();
  stream.skip_whitespace_and_comments()?;

  if let Some((Token::Grouping(GroupingType::Close(GroupingKind::Brace)), end)) = stream.peek()? {
    stream.seek();

    let span = Span::from_pair(start, end);

    let empty_block = lang::expr::BlockExpression::new_dirty(parent, span);
    let id = store.rget_mut(function).add_block(empty_block);

    return Ok(Some(lang::reference::BlockReference(function, id)));
  };

  let Some((Token::Indent(0..), _)) = indenter.peek(stream)? else {
    return stream.expected_here(line_dbg!("a newline (positive indent)"));
  };
  stream.seek();

  let block = lang::expr::BlockExpression::new_dirty(parent, start);
  let block = store.rget_mut(function).add_block(block);
  let block = lang::reference::BlockReference(function, block);

  let mut non_return_last = None;
  loop {
    stream.skip_whitespace_and_comments()?;

    let Some(stmt) = make_block_statement(store, stream, &indenter, module, function, block)? else {
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

  let children = &<C::Store<'pool> as Store<BlockReference<C>>>::rget(store, block).children;
  let returns_last = !children.is_empty() && !non_return_last.is_some_and(
    |ExpressionReference(_, id)| id == *children.last().unwrap()
  );

  let out = if returns_last {
    let &index = children.last().unwrap();
    let reference = ExpressionReference(block, index);

    lang::ty::TypeValue::Reference(
      lang::reference::TypeReference::Expression(reference)
    )
  } else {
    lang::ty::TypeValue::Intrinsic {
      kind: lang::intrinsic::Intrinsic::Void,
      span,
    }
  };

  todo!();
  // store.rget_mut(block).out = out;
  store.rget_mut(block).returns_last = returns_last;
  store.rget_mut(block).span = span;

  Ok(Some(block))
}
