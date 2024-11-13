use std::rc::Weak;

use super::*;
use crate::asterizer::ast;
use crate::compiler::{CompilerStoreHandle, CompilerWorkflow};

/// Represents an unresolved type and the information necessary to resolve it
#[allow(unused)]
#[derive(Debug)]
pub(crate) struct UnresolvedType<W: CompilerWorkflow> {
  /// The module this identifier belongs to
  pub(crate) module: CompilerStoreHandle<W>,
  /// The identifier that has yet to be resolved into a cohesive type
  pub(crate) qualified: ast::Qualified<W>,
}

/// An intrinsic type
#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Intrinsic {
  // Void type for values that can never exist
  Void,
  // An 8-bit, unsigned integer
  U8,
  // A 16-bit, unsigned integer
  U16,
  // A 32-bit, unsigned integer
  U32,
  // A 64-bit, unsigned integer
  U64,
  // An 8-bit, signed integer
  I8,
  // A 16-bit, signed integer
  I16,
  // A 32-bit, signed integer
  I32,
  // A 64-bit, signed integer
  I64,
  // A 16-bit IEEE-754 floating point value
  F16,
  // A 32-bit IEEE-754 floating point value
  F32,
  // A 64-bit IEEE-754 floating point value
  F64,
  // TODO: more?
}

/// A Type of any kind, including unresolved
#[allow(unused)]
#[derive(Debug, Clone)]
pub(crate) enum Type<S: Scope>
where
  Self: SearchIn<S>,
{
  /// An intrinsic type
  Intrinsic {
    kind: Intrinsic,
    parent: OpaqueParent<WeakCell<S>>,
  },
  TypeOfExpression {
    weak: WeakCell<Instruction>,
  },
  Intersection(RcCell<Vec<Self>>),
  UnresolvedInstrinsic(Weak<LiteralInstructionKind>),
  Reference(RcCell<Reference<Type<S>, S>>),
}

impl<S: Scope> Type<S> where Self: SearchIn<S> {
  pub(crate) fn new_intrinsic<T: Into<OpaqueParent<WeakCell<S>>>>(kind: Intrinsic, parent: T) -> Self {
    Self::Intrinsic {
      kind,
      parent: parent.into(),
    }
  }

  pub(crate) fn new_intersection<T: IntoIterator<Item = Self>>(values: T) -> Self {
    Self::Intersection(new_rc_cell(values.into_iter().collect()))
  }
}
