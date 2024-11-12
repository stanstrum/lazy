mod parse;
mod search;

use crate::enchant;

use super::*;

impl Scope for Module {
  type Index = str;
  type Part = ast::Identifier<DefaultWorkflow>;
}

impl Export {
  pub(crate) fn get_reference(&self) -> WeakCell<Reference<ModuleChild, Module>> {
    Rc::downgrade(&self.reference)
  }

  pub(crate) fn get_name(&self, compiler: &Compiler<DefaultWorkflow>) -> Result<Option<String>> {
    if let Some(name) = self.name.to_owned() {
      return Ok(Some(name));
    };

    if let Some(weak) = self.get_reference().upgrade().unwrap().get()? {
      let Some(rc) = weak.upgrade() else {
        return Ok(None);
      };

      return Ok(rc.borrow().name(compiler));
    };

    Ok(None)
  }
}

impl ModuleName {
  pub(crate) fn name(&self, compiler: &Compiler<DefaultWorkflow>) -> String {
    match self {
      ModuleName::Identifier(identifier) => identifier.name.to_owned(),
      ModuleName::File(handle) => handle.proper_name(compiler),
    }
  }
}

impl ModuleChild {
  pub(crate) fn name(&self, compiler: &Compiler<DefaultWorkflow>) -> Option<String> {
    match self {
      ModuleChild::Function(rc) => Some(rc.borrow().name.name.to_owned()),
      ModuleChild::Type(rc) => Some(rc.borrow().name.name.to_owned()),
      ModuleChild::Module(rc) => Some(rc.borrow().name.name(compiler)),
    }
  }
}
