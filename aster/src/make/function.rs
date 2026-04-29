use lang::reference::{ExpressionReference, FunctionGetBody, Reference};
use lang::span::GetSpan;
use lazy_macros::line_dbg;

use lang::token::Operator;

use super::*;

#[derive(Debug)]
pub(crate) struct TypeValueAndNamePair<C: Compiler> {
  name: lang::module::Name<C>,
  ty: lang::ty::TypeValue<C>,
  span: lang::span::Span<C>
}

impl<C: Compiler> TypeValueAndNamePair<C> {
  pub(crate) fn into_variable(self, type_reference: lang::reference::TypeReference<C>) -> lang::expr::Variable<C> {
    let Self { name, ty: type_value, span } = self;
    let ty = lang::ty::Type::new(type_reference, type_value);

    lang::expr::Variable { name, ty, span }
  }
}

pub(super) fn make_function_argument<'pool, C: Compiler, const N: usize, T: Read>(
  store: &mut C::Store<'pool>,
  stream: &mut Rereader<'pool, C, N, T>,
  parent: C::ModuleReference,
) -> Result<Option<TypeValueAndNamePair<C>>, Error<C>> {
  let Some(ty) = ty::make_type(store, stream, parent)? else {
    return Ok(None);
  };

  if !stream.skip_whitespace_and_comments()? {
    return stream.expected_here(line_dbg!("whitespace"));
  };

  let Some(name) = make_name(stream)? else {
    return stream.expected_here(line_dbg!("an identifier"));
  };

  let span = Span::from_pair(ty.get_span(store), name.span);

  Ok(Some(TypeValueAndNamePair { name, ty, span, }))
}

fn make_function_from_header<'pool, C: Compiler, const N: usize, T: Read>(
  store: &mut C::Store<'pool>,
  stream: &mut Rereader<'pool, C, N, T>,
  parent: C::ModuleReference,
) -> Result<Option<C::FunctionReference>, Error<C>> {
  let Some(name) = make_name(stream)? else {
    return Ok(None);
  };

  stream.skip_whitespace_and_comments()?;

  let ret_ty = {
    if let Some((Token::Operator(Operator::RightArrow), _)) = stream.peek()? {
      stream.seek();
      stream.skip_whitespace_and_comments()?;

      let Some(ret_ty) = ty::make_type(store, stream, parent)? else {
        return stream.expected_here(line_dbg!("a return type"));
      };

      ret_ty
    } else {
      lang::ty::TypeValue::Intrinsic {
        kind: lang::intrinsic::Intrinsic::Void,
        span: name.span,
      }
    }
  };

  stream.skip_whitespace_and_comments()?;

  let Some((Token::Indent(1..), _)) = stream.peek()? else {
    return stream.expected_here(line_dbg!("a positive indent"));
  };
  stream.seek();

  let mut arguments = vec![];
  loop {
    if let Some((Token::Indent(0), _)) = stream.peek()? {
      stream.seek();
      break;
    };

    stream.skip_whitespace_and_comments()?;

    let Some(argument) = make_function_argument(store, stream, parent)? else {
      return stream.expected_here(line_dbg!("a function argument"));
    };

    stream.skip_whitespace_and_comments()?;

    arguments.push(argument);

    let Some((Token::Indent(0), _)) = stream.peek()? else {
      return stream.expected_here(line_dbg!("a newline (0 indent)"));
    };
    stream.seek();
  };

  let mut span = name.span;
  span.end = stream.here()?.start;

  let ret_ty_span = ret_ty.get_span(store);
  let function_reference = store.create_function(parent, |function_reference| {
    let ret_ty = lang::ty::Type::new(
      lang::reference::TypeReference::ReturnTypeOf(function_reference),
      lang::ty::TypeValue::Weak {
        span: ret_ty_span,
      },
    );
    let arguments = arguments.into_iter().enumerate()
      .map(|(index, arg)| {
        let type_reference = lang::reference::TypeReference::Variable(
          lang::reference::VariableReference::Argument(function_reference, index)
        );
        arg.into_variable(type_reference)
      })
      .collect();

    lang::function::FunctionHeader {
      name,
      ret_ty,
      arguments,
      span,
    }
  });

  Ok(Some(function_reference))
}

pub(super) fn make_function<'pool, C: Compiler, const N: usize, T: Read>(
  store: &mut C::Store<'pool>,
  stream: &mut Rereader<'pool, C, N, T>,
  module: C::ModuleReference,
) -> Result<Option<C::FunctionReference>, Error<C>> {
  let Some(function) = make_function_from_header(store, stream, module)? else {
    return Ok(None);
  };

  let body = function.body();

  let mut non_return_last = None;

  let indenter = stream.indenter_here()?;

  loop {
    stream.skip_whitespace_and_comments()?;

    let Some(stmt) = expr::block::make_block_statement(
      store, stream, &indenter,
      module, function, body,
    )? else {
      return stream.expected_here(line_dbg!("a block statement"));
    };

    if stmt.non_return_last && stmt.expr.is_some() {
      non_return_last = stmt.expr;
    };

    if stmt.end {
      break;
    };
  };

  if let Some(&last) = body.rget_from(store).children.last() {
    let body_ref: &mut lang::expr::BlockExpression<C> = function.get_body_mut(store);

    body_ref.returns_last = !non_return_last.is_some_and(
      |ExpressionReference(_, id)| id == last
    );

    let expr_reference = ExpressionReference(body, last);
    body_ref.out = lang::reference::TypeReference::Expression(expr_reference).into();
  };

  function.rget_from_mut(store).span.end = stream.here()?.start;

  Ok(Some(function))
}
