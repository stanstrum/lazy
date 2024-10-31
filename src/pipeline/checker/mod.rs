use crate::compiler::{
  Compiler,
  CompilerStoreHandle,
  CompilerWorkflow,
  Check,
};

use crate::translator::lang::*;

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Checker<W: CompilerWorkflow> {
  input: Option<RcCell<Module>>,
  handle: CompilerStoreHandle<W>,
}

impl<W: CompilerWorkflow> Check<W> for Checker<W> {
  type In = RcCell<Module>;
  type Out = ();

  fn new(input: Self::In, handle: CompilerStoreHandle<W>) -> Self {
    Self {
      handle,
      input: Some(input),
    }
  }

  fn check(mut self, _compiler: &mut Compiler<W>) -> crate::Result<Self::Out> {
    let input = self.input.take().unwrap();

    dbg!(&input);

    todo!()
  }
}
