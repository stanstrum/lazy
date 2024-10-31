pub(crate) mod lang;
mod impls;

use lang::*;

use crate::compiler::workflow::DefaultWorkflow;
use crate::Result;

use crate::compiler::{
  Compiler,
  CompilerStoreHandle,
  CompilerWorkflow,
  Translate,
};

use crate::asterizer::ast::TopLevelNamespace;

trait ParseScope<'a>: Sized + SearchIn<Self::Scope> {
  type In;
  type Scope: Scope;

  fn parse_scope(translator: &mut Translator<DefaultWorkflow>, input: Self::In, parent: &Option<RcCell<Self::Scope>>) -> Result<RcCell<Self>>;
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Translator<W: CompilerWorkflow> {
  ast: Option<TopLevelNamespace<W>>,
  handle: CompilerStoreHandle<W>,
}

impl Translator<DefaultWorkflow> {
  fn parse_scope<'a, T: ParseScope<'a> + SearchIn<S>, S: Scope>(&mut self, input: T::In, parent: &Option<RcCell<T::Scope>>) -> Result<RcCell<T>> {
    T::parse_scope(self, input, parent)
  }
}

impl Translate<DefaultWorkflow> for Translator<DefaultWorkflow> {
  type In = TopLevelNamespace<DefaultWorkflow>;
  type Out = RcCell<Module>;

  fn new(ast: Self::In, handle: CompilerStoreHandle<DefaultWorkflow>) -> Self {
    Self {
      ast: Some(ast),
      handle,
    }
  }

  fn translate(mut self, _compiler: &mut Compiler<DefaultWorkflow>) -> Result<Self::Out> {
    let ast = self.ast.take().unwrap();

    let mut children = vec![];

    let module = new_rc_cell(Module {
      parent: None.into(),
      name: ModuleName::File(self.handle),
      children: vec![],
      span: ast.span,
    });

    let parent = Some(module.clone());

    for child in ast.children {
      let child = self.parse_scope::<ModuleChild, Module>(child, &parent)?;
      children.push(child);
    };

    {
      module.borrow_mut().children = children;
    };

    Ok(module)
  }
}
