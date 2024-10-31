mod module;
mod function;
mod r#type;

use super::*;
use crate::Result;

use crate::compiler::workflow::DefaultWorkflow;
use crate::asterizer::ast;

impl<S: Scope> Scope for RcCell<S> {
  type Index = S::Index;
}

impl<V: SearchIn<S>, S: Scope> Reference<V, S> {
  fn parent(&self) -> Option<RcCell<S>> {
    match self {
      Reference::Resolved(rc) => rc.borrow().parent(),
      Reference::Unresolved(rc) => Some(rc.borrow().context.clone().unwrap()),
    }
  }
}

pub(crate) trait ScopeParent<S: Scope, T: SearchIn<S>> {
  fn scope_parent(&self) -> Option<RcCell<S>>;
}

impl<S: Scope, T: SearchIn<S>> ScopeParent<S, T> for RcCell<T> {
  fn scope_parent(&self) -> Option<RcCell<S>> {
    self.borrow().parent().clone()
  }
}
