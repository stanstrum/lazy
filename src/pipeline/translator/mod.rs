use crate::Result;

use crate::compiler::{
  Compiler,
  CompilerStoreHandle,
  CompilerWorkflow,
  Translate,
};

use crate::asterizer::ast::TopLevelNamespace;

pub(crate) struct Translator<W: CompilerWorkflow> {
  ast: TopLevelNamespace<W>,
  handle: CompilerStoreHandle<W>,
}

impl<W: CompilerWorkflow> Translate<W> for Translator<W> {
  type In = TopLevelNamespace<W>;
  type Out = ();

  fn new(ast: Self::In, handle: CompilerStoreHandle<W>) -> Self {
    Self {
      ast,
      handle,
    }
  }

  fn translate(self, compiler: &mut Compiler<W>) -> Result<Self::Out> {
    todo!()
  }
}
