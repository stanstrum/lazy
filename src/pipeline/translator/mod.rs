pub(crate) mod lang;
mod impls;

use lang::Module;

use crate::Result;

use crate::compiler::{
  Compiler,
  CompilerStoreHandle,
  CompilerWorkflow,
  Translate,
};

use crate::asterizer::ast::TopLevelNamespace;

#[allow(unused)]
pub(crate) struct Translator<W: CompilerWorkflow> {
  ast: Option<TopLevelNamespace<W>>,
  handle: CompilerStoreHandle<W>,
}

impl<W: CompilerWorkflow> Translate<W> for Translator<W> {
  type In = TopLevelNamespace<W>;
  type Out = Module<W>;

  fn new(ast: Self::In, handle: CompilerStoreHandle<W>) -> Self {
    Self {
      ast: Some(ast),
      handle,
    }
  }

  fn translate(mut self, compiler: &mut Compiler<W>) -> Result<Self::Out> {
    let ast = self.ast.take().unwrap();

    let module = self.make_top_level_namespace(compiler, ast)?;

    Ok(module)
  }
}
