use crate::Compiler;

pub trait Store<Item> {
  type Out;

  fn rget(&self, key: Item) -> &Self::Out;
  fn rget_mut(&mut self, key: Item) -> &mut Self::Out;
}

pub trait Reference<S: Store<Self>>: Sized {
  fn rget_from<'a>(&self, store: &'a S) -> &'a S::Out;
  fn rget_from_mut<'a>(&self, store: &'a mut S) -> &'a mut S::Out;
}

impl<R: Copy, S: Store<R>> Reference<S> for R {
  fn rget_from<'a>(&self, store: &'a S) -> &'a S::Out {
    store.rget(*self)
  }

  fn rget_from_mut<'a>(&self, store: &'a mut S) -> &'a mut S::Out {
    store.rget_mut(*self)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypePartId(pub(crate) usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypePartReference<C: Compiler>(pub(crate) C::ModuleReference, pub(crate) TypePartId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeReference<C: Compiler> {
  Alias(C::TypeAliasReference),
  Part(TypePartReference<C>),
  Expression(C::ExpressionReference),
  Block(C::BlockReference),
  ReturnTypeOf(C::FunctionReference),
  Variable(C::VariableReference),
  StructMember(C::StructReference, usize),
}
