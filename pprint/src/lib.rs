mod ty;
mod expr;
mod block;
mod module;
mod function;

use lang::span::{GetSpan, Span};
use lang::reference::{Reference, Store};
use lang::function::Function;
use lang::{Compiler, CompilerPoolStore};

pub trait Pretty<C: Compiler> {
  type Out;

  fn print<'local, 'store, 'pool>(&'local self, store: &'store C::Store<'pool>) -> Self::Out;
}

pub trait PrettyFunction<'store, C: Compiler + 'store>: Sized + 'store where FunctionAnd<'store, C, Self>: Pretty<C> {
  fn print_with<'local, 'pool>(&'local self, function: &'store Function<C>, store: &'store C::Store<'pool>) -> <FunctionAnd<'store, C, Self> as Pretty<C>>::Out where 'local: 'store;
}

type FunctionAnd<'store, C, T> = (&'store Function<C>, &'store T);

impl<'store, C: Compiler + 'store, T: 'store> PrettyFunction<'store, C> for T where FunctionAnd<'store, C, T>: Pretty<C> {
  fn print_with<'local, 'pool>(&'local self, function: &'store Function<C>, store: &'store C::Store<'pool>) -> <FunctionAnd<'store, C, Self> as Pretty<C>>::Out where 'local: 'store {
    (function, self).print(store)
  }
}
