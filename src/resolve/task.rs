use std::collections::VecDeque;

use crate::lang::{self, Lazy};
use crate::resolve::coerce::Coerce;
use crate::resolve::reference::{BlockReference, ExpressionReference, Reference, TypeReference};

use super::Error;

pub(super) trait DoTask: std::fmt::Debug {
  fn into_task<'a>(self) -> Task<dyn DoTask + 'a> where Self: Sized + 'a {
    Task {
      this: Box::new(self),
      and_then: vec![],
    }
  }

  fn apply(self: Box<Self>, lazy: &mut Lazy, tasks: &mut Tasks) -> Result<(), Box<Error>>;
}

pub(super) type Tasks = VecDeque<Task<dyn DoTask>>;

#[derive(Debug)]
pub(super) struct Task<T: DoTask + ?Sized> {
  pub this: Box<T>,
  pub and_then: Vec<Box<dyn DoTask>>,
}

#[derive(Debug)]
pub(super) struct ReplaceType {
  pub dest: TypeReference,
  pub src: lang::ty::Type,
}

#[derive(Debug)]
pub(super) struct ResolveQualified {
  pub dest: TypeReference,
  pub reference: TypeReference,
}

#[derive(Debug)]
pub(super) struct CoerceReference<R: for<'a> Reference<'a, Out = C>, C: Coerce<R>> {
  pub dest: R,
  pub reference: TypeReference,
}

impl DoTask for ReplaceType {
  fn apply(self: Box<Self>, lazy: &mut Lazy, _tasks: &mut Tasks) -> Result<(), Box<Error>> {
    *self.dest.rget_from_mut(lazy) = self.src;

    Ok(())
  }
}

impl DoTask for ResolveQualified {
  fn apply(self: Box<Self>, lazy: &mut Lazy, _tasks: &mut Tasks) -> Result<(), Box<Error>> {
    let dest = self.dest.rget_from_mut(lazy);

    *dest = lang::ty::Type::Resolved {
      original: Box::new(dest.to_owned()),
      reference: self.reference,
    };

    Ok(())
  }
}

impl<R: for<'a, 'b> Reference<'a, Parent<'b> = Lazy<'b>, Out = C>, C: Coerce<R>> DoTask for CoerceReference<R, C> {
  fn apply(self: Box<Self>, lazy: &mut Lazy, tasks: &mut Tasks) -> Result<(), Box<Error>> {
    let dest = self.dest.rget_from(lazy);
    dest.coerce(lazy, &self.dest, &self.reference, tasks)
  }
}
