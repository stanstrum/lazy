pub(crate) mod variable_resolution;

use std::cmp::Ordering;

use lazy_macros::print_message;
use lang::span::{GetSpan, Span};
use lang::token::{GroupingKind, GroupingType, Operator};
use lang::reference::{BlockReference, ExpressionReference, Reference, Store};
use lang::Compiler;

use crate::make::Indenter;

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
    expr.map(|b| {
      let variable_span = (&*store).rget(variable).span;

      let span = b.rget_from(store).get_span(store);

      let a_expr = lang::expr::Expression::Variable { reference: variable, span: variable_span };
      let a_expr_index = function.rget_from_mut(store).add_expr(a_expr);
      let a = ExpressionReference(block, a_expr_index);

      lang::expr::BlockExpression::create_new_expr_in(store, block, |expr_reference| {

        let type_reference = lang::reference::TypeReference::Expression(expr_reference);
        let type_value = lang::ty::TypeValue::Weak { span };
        let out = lang::ty::Type::new(type_reference, type_value);

        lang::expr::Expression::Binary {
          a,
          b,
          op: (lang::expr::operator::BinaryOperator::Assign, variable_span),
          span,
          out,
        }
      })
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

    let empty_block = lang::expr::BlockExpression::create_in(
      store,
      function, parent, span,
      lang::ty::TypeValue::Weak { span },
    );

    return Ok(Some(empty_block));
  };

  let Some((Token::Indent(0..), _)) = indenter.peek(stream)? else {
    return stream.expected_here(line_dbg!("a newline (positive indent)"));
  };
  stream.seek();

  let block = lang::expr::BlockExpression::create_in(
    store,
    function, parent, start,
    lang::ty::TypeValue::Weak { span: start },
  );

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
