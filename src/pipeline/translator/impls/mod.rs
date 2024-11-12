mod module;
mod function;
mod r#type;

use super::*;
use crate::Result;

use crate::compiler::workflow::DefaultWorkflow;
use crate::asterizer::ast;

pub(crate) trait ReferenceResolve<V: SearchIn<S>, S: Scope> {
  fn parent(&self) -> Option<WeakCell<S>>;
  fn get(&self) -> Result<Option<WeakCell<V>>>;
}

impl<V: SearchIn<S>, S: Scope> ReferenceResolve<V, S> for Reference<V, S> {
  fn parent(&self) -> Option<WeakCell<S>> {
    match self {
      Reference::Resolved(rc) => rc.try_borrow().unwrap().parent(),
      Reference::Unresolved(rc) => Some(rc.try_borrow().unwrap().context.clone().unwrap()),
    }
  }

  fn get(&self) -> Result<Option<WeakCell<V>>> {
    Ok(match self {
      Reference::Resolved(reference) => Some(Rc::downgrade(reference)),
      Reference::Unresolved(rc) => {
        if let ScopeSearch::Found(weak) = rc.borrow().find_reference()? {
          Some(weak)
        } else {
          None
        }
      },
    })
  }
}

impl<V: SearchIn<S>, S: Scope> ReferenceResolve<V, S> for RcCell<Reference<V, S>> {
  fn parent(&self) -> Option<WeakCell<S>> {
    self.borrow().parent()
  }

  fn get(&self) -> Result<Option<WeakCell<V>>> {
    self.borrow().get()
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

  fn parse_scope(_translator: &mut Translator<DefaultWorkflow>, _compiler: &Compiler<DefaultWorkflow>, _input: Self::In, _parent: &Option<WeakCell<Self::Scope>>) -> Result<RcCell<Self>> {
    todo!()
  }
}

impl SearchIn<FunctionBlock> for Instruction {
  fn parent(&self) -> Option<WeakCell<FunctionBlock>> {
    todo!()
  }

  fn search_in(_scope: &FunctionBlock, _index: &<FunctionBlock as Scope>::Index) -> Result<ScopeSearch<Self, FunctionBlock>> {
    todo!()
  }
}
