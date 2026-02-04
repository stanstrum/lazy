use crate::aster::pprint::Pretty;
use crate::lang::span::GetSpan;
use crate::lang::{self, Lazy};
use crate::line_dbg;
use crate::resolve::reference::{ExpressionReference, Reference, TypeReference};

use super::Error;

pub fn type_of_expect(lazy: &Lazy, expression: &ExpressionReference) -> Result<lang::ty::Type, Box<Error>> {
  let Some(ty) = type_of(lazy, expression)? else {
    let function = &lazy[expression.function];
    let at = expression.rget_from(lazy).get_span(function);

    return Err(Box::new(Error::Unresolved {
      what: line_dbg!("expression"),
      at,
    }));
  };

  Ok(ty)
}

pub fn type_of(lazy: &Lazy, expression: &ExpressionReference) -> Result<Option<lang::ty::Type>, Box<Error>> {
  match expression.rget_from(lazy) {
    lang::expr::Expression::BlockExpression(block_id) => {
      let block = &lazy[expression.function][*block_id];

      if !block.returns_last {
        Ok(Some(lang::ty::Type::Intrinsic {
          kind: lang::ty::Intrinsic::Void,
          span: block.span,
        }))
      } else {
        let &index = block.children.last().unwrap();
        type_of(lazy, &ExpressionReference { function: expression.function, index })
      }
    },
    lang::expr::Expression::Literal { out, .. } => {
      Ok(Some(out.to_owned()))
    },
  }
}
