use std::{
  cell::RefCell,
  rc::{Rc, Weak},
  fmt::Debug,
};

use super::*;
use crate::Result;

use ast::{Identifier, Qualified};

pub(crate) type RcCell<T> = Rc<RefCell<T>>;

#[allow(unused)]
pub(crate) enum ScopeSearch<V: SearchIn<S>, S: Scope> {
  Found(RcCell<V>),
  Next(RcCell<S>),
  None,
}

#[allow(unused)]
pub(crate) trait SearchIn<S: Scope>: Sized + Debug {
  fn parent(&self) -> Option<RcCell<S>>;
  fn search_in(scope: &S, index: &S::Index) -> Result<ScopeSearch<Self, S>>;
}

#[allow(unused)]
pub(crate) trait Scope: Debug + Sized {
  type Index: Debug + ?Sized;

  fn search<I: SearchIn<Self>>(&self, index: &Self::Index) -> Result<ScopeSearch<I, Self>> {
    I::search_in(self, index)
  }
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct UnresolvedReference<S: Scope, W: CompilerWorkflow = DefaultWorkflow> {
  pub(crate) context: OpaqueParent<RcCell<S>>,
  implicit: bool,
  parts: Vec<Identifier<W>>,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) enum Reference<V: SearchIn<S>, S: Scope> {
  Resolved(RcCell<V>),
  Unresolved(RcCell<UnresolvedReference<S>>),
}

impl<S: Scope> Type<S> where Self: SearchIn<S> {
  pub(crate) fn new_unknown(context: &RcCell<S>, qualified: Qualified<DefaultWorkflow>) -> RcCell<Self> {
    new_rc_cell(Self::Reference(Reference::Unresolved(new_rc_cell(UnresolvedReference {
      context: context.clone().into(),
      implicit: qualified.implicit,
      parts: qualified.parts,
    }))))
  }
}

pub(crate) fn new_rc_cell<T>(value: T) -> RcCell<T> {
  Rc::new(RefCell::new(value))
}

impl<S: Scope<Index = str>> UnresolvedReference<S> {
  pub(crate) fn find_reference<V: SearchIn<S>>(&self) -> Result<ScopeSearch<V, S>> {
    let mut context = Rc::downgrade(&self.context.parent);

    for part in self.parts.iter() {
      let search = {
        trace!("borrow UnresolvedReference context via weak upgrade");

        context.upgrade().unwrap().borrow_mut().search::<V>(&part.name)?
      };

      let next = match search {
        ScopeSearch::Found(rc) => return Ok(ScopeSearch::Found(rc.clone())),
        ScopeSearch::Next(rc) => Rc::downgrade(&rc),
        ScopeSearch::None => return Ok(ScopeSearch::None),
      };

      context = next;
    };

    todo!()
  }
}
