use std::marker::PhantomData;

use crate::Result;
use crate::compiler::*;

use crate::translator::lang::{
  Module,
  RcCell,
};

macro_rules! make_todo_stage {
  ($name:ident: $trait:ident::<In = $in:ty>::$method:ident) => {
    make_todo_stage!(@inner: $name, $trait, $method, In = $in);
  };

  ($name:ident: $trait:ident::<In = $in:ty, Out = $out:ty>::$method:ident) => {
    make_todo_stage!(@inner: $name, $trait, $method, In = $in, Out = $out);
  };

  (@inner: $name:ident, $trait:ident, $method:ident, $($assoc:ident = $ty:ty),+) => {
    #[derive(Debug)]
    pub(super) struct $name<W: CompilerWorkflow> {
      marker: PhantomData<W>,
    }

    impl<W: CompilerWorkflow> $trait<W> for $name<W> {
      $(type $assoc = $ty;)+

      fn new(_input: Self::In, _handle: CompilerStoreHandle<W>) -> Self {
        Self {
          marker: Default::default(),
        }
      }

      fn $method(self, _compiler: &mut Compiler<W>) -> Result {
        todo!()
      }
    }
  };
}

// These allow for making skeletons for the compiler workflow without fully
// implementing each stage
make_todo_stage! { Generator: Generate::<In = (), Out = ()>::generate }
make_todo_stage! { Outputter: Output::<In = ()>::output }
