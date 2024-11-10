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
  fn parent(&self) -> Option<WeakCell<S>> {
    match self {
      Reference::Resolved(rc) => rc.try_borrow().unwrap().parent(),
      Reference::Unresolved(rc) => Some(rc.try_borrow().unwrap().context.clone().unwrap()),
    }
  }
}

pub(crate) trait ScopeParent<S: Scope, T: SearchIn<S>> {
  fn scope_parent(&self) -> Option<WeakCell<S>>;
}

impl<S: Scope, T: SearchIn<S>> ScopeParent<S, T> for RcCell<T> {
  fn scope_parent(&self) -> Option<WeakCell<S>> {
    self.try_borrow().unwrap().parent().clone()
  }
}

impl<'a> ParseScope<'a> for Instruction {
  type In = ast::BlockChild<DefaultWorkflow>;
  type Scope = FunctionBlock;

  fn parse_scope(translator: &mut Translator<DefaultWorkflow>, input: Self::In, parent: &Option<WeakCell<Self::Scope>>) -> Result<RcCell<Self>> {
    todo!()
  }
}

impl SearchIn<FunctionBlock> for Instruction {
  fn parent(&self) -> Option<WeakCell<FunctionBlock>> {
    todo!()
  }

  fn search_in(scope: &FunctionBlock, index: &<FunctionBlock as Scope>::Index) -> Result<ScopeSearch<Self, FunctionBlock>> {
    todo!()
  }
}
