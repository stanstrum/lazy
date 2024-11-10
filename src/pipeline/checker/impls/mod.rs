use std::rc::Weak;

use crate::translator::ReferenceResolve;

use super::*;

mod module;
mod function;

pub(crate) use module::*;

impl<V: SearchIn<S>, S: Scope> Resolve for RcCell<Reference<V, S>> {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    self.get_and_maybe_modify(mods)?;

    ok
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    let this = self.borrow();

    let Reference::Resolved(_) = &*this else {
      let span = this.get_inner_weak().as_ref().and_then(Weak::upgrade).unwrap().borrow().span(compiler);
      return UnresolvedQualifiedSnafu { span }.fail()?;
    };

    ok
  }
}

impl GetAndMaybeModify<Type<Module>, Module> for RcCell<Type<Module>> {
  fn get_and_maybe_modify(&self, mods: &mut Modifications) -> Result<Option<WeakCell<Type<Module>>>> {
    todo!()
  }
}

impl Resolve for RcCell<Type<Module>> {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    match &*self.borrow() {
      Type::Intrinsic { .. } => ok,
      Type::Reference(reference) => reference.resolve(mods)
    }
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    match &*self.borrow() {
      Type::Intrinsic { .. } => ok,
      Type::Reference(reference) => match &*reference.borrow() {
        Reference::Resolved(rc) => rc.ensure_resolved(compiler),
        Reference::Unresolved(_) => todo!(),
      }
    }
  }
}
