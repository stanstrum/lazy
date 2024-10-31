use crate::Result;

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
  input: RcCell<Module>,
  handle: CompilerStoreHandle<W>,
}

trait Resolve: Sized {
  fn resolve(&mut self) -> Result<bool>;
}

impl Resolve for Function {
  fn resolve(&mut self) -> Result<bool> {
    todo!()
  }
}

impl Resolve for ModuleChild {
  fn resolve(&mut self) -> Result<bool> {
    match self {
      ModuleChild::Function(rc) => rc.borrow_mut().resolve(),
      ModuleChild::Module(rc) => rc.borrow_mut().resolve(),
    }
  }
}

impl Resolve for Module {
  fn resolve(&mut self) -> Result<bool> {
    let mut did_work = false;

    for child in self.children.iter_mut() {
      did_work |= child.borrow_mut().resolve()?;
    };

    Ok(did_work)
  }
}

impl<W: CompilerWorkflow> Check<W> for Checker<W> {
  type In = RcCell<Module>;
  type Out = RcCell<Module>;

  fn new(input: Self::In, handle: CompilerStoreHandle<W>) -> Self {
    Self {
      handle,
      input,
    }
  }

  fn check(self, _compiler: &mut Compiler<W>) -> Result<Self::Out> {
    trace!("{:#?}", &self.input);

    let mut counter = 1;
    loop {
      trace!("check: resolve pass #{counter}");

      if !self.input.borrow_mut().resolve()? {
        break;
      };

      counter += 1;
    };

    Ok(self.input)
  }
}
