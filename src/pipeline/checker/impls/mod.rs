mod function;
mod instruction;
mod module;
mod ty;

use std::rc::Weak;

use super::*;
use crate::translator::ScopeParent;

pub(crate) use module::*;

impl<C: CoerceWith<T>, T> CoerceWith<RcCell<T>> for C {
  fn coerce_with(&self, with: &RcCell<T>, mods: &mut Modifications) -> Result {
    self.coerce_with(&*with.borrow(), mods)
  }
}

impl<V: SearchIn<S>, S: Scope> Resolve for RcCell<Reference<V, S>> {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    self.get_and_maybe_modify(mods)?;
    ok
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    let this = self.borrow();

    let Reference::Resolved(_) = &*this else {
      let span = this
        .get_inner_weak()
        .as_ref()
        .and_then(Weak::upgrade)
        .unwrap()
        .borrow()
        .span(compiler);
      return UnresolvedQualifiedSnafu { span }.fail()?;
    };

    ok
  }
}
