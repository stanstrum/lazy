use crate::lang::function::ExprId;
use crate::lang::module::FunctionId;
use crate::lang::span::GetSpan;
use crate::lang::{self, Lazy};
use crate::line_dbg;
use crate::resolve::reference::{Reference, TypeReference};

use super::Error;

pub trait TypeOf {
  fn type_of(&self, lazy: &Lazy) -> Result<Option<lang::ty::Type>, Box<Error>>;
}

pub fn is_assignable(lazy: &Lazy, what: &TypeReference, ty: &lang::ty::Type) -> Result<Option<bool>, Box<Error>> {
  match what.rget_from(lazy) {
    lang::ty::Type::Unresolved { .. } => Ok(None),
    _ => Ok(Some(false)),
  }
}

pub fn type_of_expect(lazy: &Lazy, function: FunctionId, expr: ExprId) -> Result<lang::ty::Type, Box<Error>> {
  let Some(ty) = type_of(lazy, function, expr)? else {
    let function = &lazy[function];
    let at = function[expr].get_span(function);

    return Err(Box::new(Error::Unresolved {
      what: line_dbg!("expression"),
      at,
    }));
  };

  Ok(ty)
}

pub fn type_of(lazy: &Lazy, function: FunctionId, expr: ExprId) -> Result<Option<lang::ty::Type>, Box<Error>> {
  match &lazy[function][expr] {
    lang::expr::Expression::BlockExpression(block_id) => {
      let block = &lazy[function][*block_id];

      if !block.returns_last {
        Ok(Some(lang::ty::Type::Intrinsic {
          kind: lang::ty::Intrinsic::Void,
          span: block.span,
        }))
      } else {
        let last_expr = block.children.last().unwrap();
        type_of(lazy, function, *last_expr)
      }
    },
    lang::expr::Expression::Literal { out, .. } => {
      Ok(Some(out.to_owned()))
    },
  }
}
