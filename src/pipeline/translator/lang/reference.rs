use std::{
  cell::RefCell,
  fmt::Debug,
  rc::{Rc, Weak},
};

use super::*;
use crate::compiler::{error::ReadSpan, Compiler};
use crate::checker::MakeModification;
use crate::{enchant, Result};

pub(crate) type RcCell<T> = Rc<RefCell<T>>;
pub(crate) type WeakCell<T> = Weak<RefCell<T>>;

#[allow(unused)]
#[derive(Debug)]
pub(crate) enum ScopeSearch<V: SearchIn<S>, S: Scope> {
  Found(WeakCell<V>),
  Next(WeakCell<S>),
  None,
}

#[allow(unused)]
pub(crate) trait SearchIn<S: Scope>:
  Sized + Debug + MakeModification<S>
{
  fn parent(&self) -> Option<WeakCell<S>> {
    todo!()
  }
  fn search_in(scope: &S, index: &S::Index) -> Result<ScopeSearch<Self, S>> {
    todo!()
  }
  fn span(&self, compiler: &Compiler<DefaultWorkflow>) -> ReadSpan {
    todo!()
  }
}

pub(crate) trait Part<S: Scope> {
  fn part_to_index(&self) -> &S::Index;
}

impl<T: Part<S>, S: Scope> Part<S> for RcCell<T> {
  fn part_to_index(&self) -> &<S as Scope>::Index {
    todo!()
  }
}

#[allow(unused)]
pub(crate) trait Scope: Debug + Sized {
  type Index: Debug + ?Sized;
  type Part: Debug + Part<Self>;

  fn search<I: SearchIn<Self>>(&self, index: &Self::Index) -> Result<ScopeSearch<I, Self>> {
    I::search_in(self, index)
  }
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct UnresolvedReference<S: Scope, W: CompilerWorkflow = DefaultWorkflow> {
  pub(crate) context: OpaqueParent<WeakCell<S>>,
  pub(crate) parts: Vec<S::Part>,
  pub(crate) implicit: bool,
  pub(crate) span: Span<W>,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) enum Reference<V: SearchIn<S>, S: Scope> {
  Resolved(RcCell<V>),
  Unresolved(RcCell<UnresolvedReference<S>>),
}

impl<V: SearchIn<S>, S: Scope> Clone for Reference<V, S> {
  fn clone(&self) -> Self {
    match self {
      Self::Resolved(rc) => Self::Resolved(rc.clone()),
      Self::Unresolved(rc) => Self::Unresolved(rc.clone()),
    }
  }
}

impl<S: Scope> Type<S>
where
  Self: SearchIn<S>,
{
  // pub(crate) fn new_unknown(context: &RcCell<S>, qualified:
  // Qualified<DefaultWorkflow>) -> RcCell<Self> {
  //   new_rc_cell(Self::Reference(Reference::Unresolved(new_rc_cell(UnresolvedReference {
  //     context: Rc::downgrade(context).into(),
  //     implicit: qualified.implicit,
  //     parts: qualified.parts,
  //   }))))
  // }
}

#[allow(unused)]
impl<V: SearchIn<S>, S: Scope> Reference<V, S> {
  pub(crate) fn new(value: V) -> Self {
    Self::Resolved(new_rc_cell(value))
  }

  pub(crate) fn get_inner_weak(&self) -> Option<WeakCell<V>> {
    match self {
      Reference::Resolved(rc) => Some(Rc::downgrade(rc)),
      Reference::Unresolved(rc) => None,
    }
  }
}

pub(crate) fn new_rc_cell<T>(value: T) -> RcCell<T> {
  Rc::new(RefCell::new(value))
}

impl<S: Scope> UnresolvedReference<S> {
  pub(crate) fn find_reference<V: SearchIn<S>>(&self) -> Result<ScopeSearch<V, S>> {
    let context = self.context.as_ref().clone();

    if self.implicit {
      trace!(
        "{}: won't resolve an implicit unknown reference",
        enchant!("find_reference")
      );
      return Ok(ScopeSearch::None);
    };

    let mut search = ScopeSearch::Next(context);
    for part in self.parts.iter().map(S::Part::part_to_index) {
      trace!(
        "{}: search for {part:?} in scope: {search:?}",
        enchant!("find_reference")
      );

      let ScopeSearch::Next(weak) = search else {
        // cannot search any other variant of ScopeSearch as a Scope
        warn!(
          "{}: tried to search in something other than a scope (likely an error)",
          enchant!("find_reference")
        );
        return Ok(ScopeSearch::None);
      };
      search = weak.upgrade().unwrap().borrow().search(part)?;
    }

    Ok(search)
  }
}
