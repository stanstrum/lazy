use crate::lang::Lazy;
use crate::lang::ty::Type;
use crate::lang::reference::{Reference, Store, TypeReference};

use super::*;
pub trait TypeOf {
  fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>>;
}

impl<R: Copy> TypeOf for R
  where for<'a> Lazy<'a>: Store<R>,
        for<'a> <Lazy<'a> as Store<R>>::Out: TypeOf
{
  fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>> {
    lazy.rget(*self).type_of(lazy)
  }
}

impl TypeOf for &Type {
  fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>> {
    match self {
      Type::Unresolved { module, qualified } => Ok(None),
      Type::Intrinsic { kind, span } => todo!(),
      Type::WeakInteger { span } => todo!(),
      Type::WeakFloat { span } => todo!(),
      Type::WeakString { span } => todo!(),
      Type::ReferenceTo { ty, r#mut, span } => todo!(),
      Type::UnsizedArrayOf { ty, span } => todo!(),
      Type::SizedArrayOf { ty, size, span } => todo!(),
      Type::Expression(expression_reference) => todo!(),
      Type::Reference(_) => todo!(),
    }
  }
}

impl TypeOf for TypeReference {
  fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>> {
    match self {
      TypeReference::Part(part) => {
        part.rget_from(lazy).type_of(lazy)
      },
      TypeReference::ReturnTypeOf(function) => {
        (&function.rget_from(lazy).header.ret_ty).type_of(lazy)
      },
      TypeReference::Alias(_) => todo!(),
      TypeReference::ArgumentOf(..) => todo!(),
    }
  }
}
