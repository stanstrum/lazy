use crate::resolve::r#typeof::type_of_expect;

use super::*;

fn verify_type(lazy: &Lazy, reference: &TypeReference) -> Result<(), Box<Error>> {
  let ty = reference.rget_from(lazy);

  match ty {
    lang::ty::Type::Unresolved { .. } => Err(Box::new(Error::Unresolved {
      what: line_dbg!("type"),
      at: ty.get_span(lazy),
    })),
    lang::ty::Type::Intrinsic { .. } => Ok(()),
    lang::ty::Type::Resolved { reference, .. } => verify_type(lazy, reference),
    lang::ty::Type::WeakInteger { span } => Err(Box::new(Error::Unresolved {
      what: line_dbg!("integer"),
      at: *span,
    })),
    lang::ty::Type::WeakFloat { span } => Err(Box::new(Error::Unresolved {
      what: line_dbg!("float"),
      at: *span,
    })),
    lang::ty::Type::ReferenceTo { .. } => {
      verify_type(lazy, &TypeReference::Dereference(Box::new(reference.to_owned())))
    },
    lang::ty::Type::SizedArrayOf { .. } => {
      todo!()
    },
    lang::ty::Type::UnsizedArrayOf { .. } => {
      todo!()
    },
    lang::ty::Type::Reference { .. } => {
      todo!()
    },
  }
}

fn verify_expr(lazy: &Lazy, reference: ExpressionReference) -> Result<(), Box<Error>> {
  match reference.rget_from(lazy) {
    lang::expr::Expression::BlockExpression(block) => verify_block(lazy, reference.function, *block),
    lang::expr::Expression::Literal { .. } => Ok(()),
  }
}

fn verify_block(lazy: &Lazy, function: lang::module::FunctionId, block: lang::function::BlockId) -> Result<(), Box<Error>> {
  let children = lazy[function][block].children.clone();

  for index in children {
    verify_expr(lazy, ExpressionReference { function, index })?;
  };

  Ok(())
}

fn verify_function(lazy: &Lazy, function: lang::module::FunctionId) -> Result<(), Box<Error>> {
  let ret_ty = TypeReference::ReturnTypeOf(function);
  verify_type(lazy, &ret_ty)?;

  let arguments_count = lazy[function].header.arguments.len();
  for index in 0..arguments_count {
    verify_type(lazy, &TypeReference::ArgumentOf { function, index })?;
  };

  let function_ref = &lazy[function];
  let body = function_ref.body;
  verify_block(lazy, function, body)?;

  // let last_span = function_ref[body].children.last()
  //   .map(|child| function_ref[*child].get_span(function_ref))
  //   .unwrap_or(function_ref[body].span);

  if !function_ref[body].returns_last {
    coerce::assert_assignable(lazy, &ret_ty, &lang::ty::Type::Intrinsic {
      kind: lang::ty::Intrinsic::Void,
      span: function_ref.header.ret_ty.get_span(lazy),
    })?;
  } else {
    let last_expr = function_ref[body].children.last().unwrap();
    let expr_type = type_of_expect(lazy, function, *last_expr)?;

    coerce::assert_assignable(lazy, &ret_ty, &expr_type)?;
  };

  Ok(())
}

pub(super) fn verify_module(lazy: &Lazy, module: lang::module::ModuleId) -> Result<(), Box<Error>> {
  for index in 0..lazy[module].aliases.len() {
    verify_type(lazy, &TypeReference::Alias { module, index })?;
  };

  for &module in lazy[module].modules.iter() {
    verify_module(lazy, module)?;
  };

  for &function in lazy[module].functions.iter() {
    verify_function(lazy, function)?;
  };

  Ok(())
}
