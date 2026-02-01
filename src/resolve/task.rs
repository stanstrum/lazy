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
pub(super) struct ResolveBlockReturn {
  pub reference: BlockReference,
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

impl DoTask for ResolveBlockReturn {
  fn apply(self: Box<Self>, lazy: &mut Lazy, tasks: &mut Tasks) -> Result<(), Box<Error>> {
    let block_ref = self.reference.rget_from_mut(lazy);

    if block_ref.out.is_none() {
      block_ref.out = Some(if dbg!(&block_ref).returns_last {
        let &index = block_ref.children.last().unwrap();
        let reference = ExpressionReference { function: self.reference.function, index };

        lang::ty::Type::Reference(TypeReference::Expression(reference))
      } else {
        lang::ty::Type::Intrinsic {
          kind: lang::ty::Intrinsic::Void,
          span: block_ref.span,
        }
      });
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
