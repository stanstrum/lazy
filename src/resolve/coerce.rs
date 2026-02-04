use crate::aster::pprint::Pretty;
use crate::lang::expr::{BlockExpression, Expression};
use crate::lang::ty::Type;

use super::*;

pub fn is_assignable(lazy: &Lazy, what: &TypeReference, ty: &lang::ty::Type) -> Result<Option<bool>, Box<Error>> {
  match (what.rget_from(lazy), ty) {
    (lang::ty::Type::Unresolved { .. }, _) => Ok(Some(true)),
    (lang::ty::Type::Intrinsic { kind: kind_a, .. }, lang::ty::Type::Intrinsic { kind: kind_b, .. }) if kind_a == kind_b => {
      Ok(Some(true))
    },
    (lang::ty::Type::Intrinsic { kind, .. }, lang::ty::Type::WeakInteger { .. }) if *kind != lang::ty::Intrinsic::Void => {
      Ok(Some(true))
    },
    (lang::ty::Type::Expression(expr), _) => {
      expr.rget_from(lazy);

      todo!()
    },
    _ => Ok(Some(false)),
  }
}

pub(super) fn assert_assignable(lazy: &Lazy, what: &TypeReference, ty: &lang::ty::Type) -> Result<(), Box<Error>> {
  match is_assignable(lazy, what, ty)? {
    Some(true) => Ok(()),
    Some(false) => Err(Box::new(Error::Incompatible {
      what: what.print(lazy),
      what_span: what.get_span(lazy),
      to: ty.print(lazy),
      to_span: ty.get_span(lazy),
    })),
    None => Err(Box::new(Error::Unresolved {
      what: "type",
      at: ty.get_span(lazy),
    })),
  }
}

pub(super) trait IsResolved {
  fn is_resolved(&self, lazy: &Lazy) -> Result<bool, Box<Error>>;
}

impl IsResolved for TypeReference {
  fn is_resolved(&self, lazy: &Lazy) -> Result<bool, Box<Error>> {
    self.rget_from(lazy).is_resolved(lazy)
  }
}

impl IsResolved for Type {
  fn is_resolved(&self, lazy: &Lazy) -> Result<bool, Box<Error>> {
    match self {
      Type::Reference(reference) => reference.is_resolved(lazy),
      Type::Intrinsic { kind, span } => Ok(true),
      Type::Resolved { reference, .. } => reference.is_resolved(lazy),
      | Type::ReferenceTo { ty, .. }
      | Type::UnsizedArrayOf { ty, .. }
      | Type::SizedArrayOf { ty, .. } => ty.is_resolved(lazy),
      Type::Expression(expression) => Ok(r#typeof::type_of(lazy, expression)?.is_some()),
      Type::Unresolved { .. } => Ok(false),
      other => todo!("{other:#?}"),
    }
  }
}

impl IsResolved for Expression {
  fn is_resolved(&self, lazy: &Lazy) -> Result<bool, Box<Error>> {
    todo!()
  }
}

impl IsResolved for BlockExpression {
  fn is_resolved(&self, lazy: &Lazy) -> Result<bool, Box<Error>> {
    self.out.is_resolved(lazy)
  }
}

pub(super) trait Coerce<R: for<'a> Reference<'a, Out = Self>>: std::fmt::Debug {
  fn coerce(&self, lazy: &Lazy, reference: &R, to: &TypeReference, tasks: &mut Tasks) -> Result<(), Box<Error>>;
}

impl Coerce<TypeReference> for Type {
  fn coerce(&self, lazy: &Lazy, reference: &TypeReference, to: &TypeReference, tasks: &mut Tasks) -> Result<(), Box<Error>> {
    assert_assignable(lazy, reference, to.rget_from(lazy))?;

    let ty = reference.rget_from(lazy);

    if !ty.is_resolved(lazy)? && to.is_resolved(lazy)? {
      tasks.push_back(task::ReplaceType {
        dest: to.to_owned(),
        src: ty.to_owned(),
      }.into_task());
    };

    Ok(())
  }
}

impl Coerce<ExpressionReference> for Expression {
  fn coerce(&self, lazy: &Lazy, reference: &ExpressionReference, to: &TypeReference, tasks: &mut Tasks) -> Result<(), Box<Error>> {
    match (self, to.rget_from(lazy)) {
      (&Expression::BlockExpression(block), _) => {
        let block_reference = BlockReference { function: reference.function, block };
        let block = block_reference.rget_from(lazy);
        block.out.coerce(lazy, &TypeReference::Block(block_reference).to_owned(), to, tasks)?;
        block.coerce(lazy, &block_reference, to, tasks)
      },
      (Expression::Literal { .. }, _) => {
        assert_assignable(lazy, &TypeReference::Expression(reference.to_owned()), to.rget_from(lazy))?;

        todo!()
      },
    }
  }
}

impl Coerce<BlockReference> for BlockExpression {
  fn coerce(&self, lazy: &Lazy, reference: &BlockReference, to: &TypeReference, tasks: &mut Tasks) -> Result<(), Box<Error>> {
    if !to.is_resolved(lazy)? {
      return Ok(());
    };

    let block_ty = TypeReference::Block(reference.to_owned());

    if !self.out.is_resolved(lazy)? {
      self.out.coerce(lazy, &block_ty, to, tasks)?;

      return Ok(());
    };

    assert_assignable(lazy, &block_ty, to.rget_from(lazy))?;

    if let Some(last) = reference.get_return_last(lazy) {
      let expression = last.rget_from(lazy);
      expression.coerce(lazy, &last, to, tasks)?;
    };

    Ok(())
  }
}
