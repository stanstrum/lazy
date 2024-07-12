use crate::tokenizer::{
  Literal,
  LiteralKind,
  GetSpan,
};

use crate::typechecker::lang::intrinsics::Intrinsic;
use crate::typechecker::{
  TypeChecker,
  check::type_of::TypeOf,
  error::*,
};

use crate::typechecker::lang::{
  Instruction,
  VariableReference,
  Value,
  Type,
  TypeCell,
  pretty_print::PrettyPrint,
};

pub(super) trait Coerce {
  fn coerce(&mut self, checker: &mut TypeChecker, to: &Type) -> Result<bool, TypeCheckerError>;
}

pub(super) trait CoerceCell {
  fn coerce_cell(&mut self, checker: &mut TypeChecker, to: &TypeCell) -> Result<bool, TypeCheckerError>;
}

impl<T: Coerce> CoerceCell for T {
  fn coerce_cell(&mut self, checker: &mut TypeChecker, to: &TypeCell) -> Result<bool, TypeCheckerError> {
    self.coerce(checker, &*to.borrow())
  }
}

pub(crate) trait Extends where Self: std::fmt::Debug + crate::typechecker::lang::pretty_print::PrettyPrint + GetSpan {
  fn extends(&self, other: &Self) -> bool;
  fn assert_extends(&self, other: &Self) -> Result<(), TypeCheckerError> {
    if self.extends(other) {
      Ok(())
    } else {
      IncompatibleTypesSnafu {
        message: "assertion failed",
        lhs: self.pretty_print(),
        rhs: other.pretty_print(),
        span: self.get_span().to_owned(),
      }.fail()
    }
  }
}

impl Extends for Type {
  fn extends(&self, other: &Self) -> bool {
    match (self, &other) {
      (Type::Unknown { .. }, _) => true,
      (Type::ReferenceTo {
        ty,
        ..
      }, Type::FuzzyString { size, element_ty, span }) => {
        let element_ty: TypeCell = Type::Intrinsic {
          kind: element_ty.to_owned(),
          span: span.to_owned(),
        }.into();

        if !ty.is_resolved() {
          return true;
        };

        if ty.extends(&Type::UnsizedArrayOf {
          ty: element_ty.to_owned(),
          span: span.to_owned(),
        }.into()) {
          return true;
        };

        // TODO: FIXME: bad bad not good
        if ty.extends(&Type::SizedArrayOf {
          count: Value::Literal {
            literal: Literal {
              kind: LiteralKind::Integer(*size as u64),
              span: span.to_owned(),
            },
            ty: Type::FuzzyInteger {
              span: span.to_owned()
            }.into(),
          },
          ty: element_ty,
          span: span.to_owned(),
        }.into()) {
          return true;
        };

        false
      },
      (Type::FuzzyString { size: lhs_size, element_ty: lhs_ty, .. }, Type::FuzzyString { size: rhs_size, element_ty: rhs_ty, .. }) => {
        if *lhs_size != *rhs_size {
          return false;
        };

        if lhs_ty != rhs_ty {
          return false;
        };

        true
      },
      (
        Type::FuzzyInteger { .. },
        Type::Intrinsic {
          kind: Intrinsic::U8
            | Intrinsic::I8
            | Intrinsic::U16
            | Intrinsic::I16
            | Intrinsic::U32
            | Intrinsic::I32
            | Intrinsic::U64
            | Intrinsic::I64,
          ..
        }
      ) => true,
      _ => false,
    }
  }
}

impl GetSpan for TypeCell {
  fn get_span(&self) -> crate::tokenizer::Span {
    self.borrow().get_span()
  }
}

impl Extends for TypeCell {
  fn extends(&self, other: &Self) -> bool {
    self.borrow().extends(&*other.borrow())
  }
}

impl Coerce for TypeCell {
  fn coerce(&mut self, _checker: &mut TypeChecker, to: &Type) -> Result<bool, TypeCheckerError> {
    let inner = &mut *self.borrow_mut();

    inner.assert_extends(to)?;
    *inner = to.to_owned();

    Ok({
      match (&inner, to) {
        (Type::Unknown { .. }, _) => true,
        (lhs, rhs) if std::mem::discriminant(*lhs) == std::mem::discriminant(rhs) => false,
        _ => {
          return IncompatibleTypesSnafu {
            message: "not coercible",
            lhs: inner.pretty_print(),
            rhs: to.pretty_print(),
            span: inner.get_span(),
          }.fail();
        }
      }
    })
  }
}

impl Coerce for VariableReference {
  fn coerce(&mut self, checker: &mut TypeChecker, to: &Type) -> Result<bool, TypeCheckerError> {
    self.get().ty.coerce(checker, to)
  }
}

impl Coerce for Instruction {
  fn coerce(&mut self, checker: &mut TypeChecker, to: &Type) -> Result<bool, TypeCheckerError> {
    match self {
      | Instruction::Return { .. }
      | Instruction::Assign { .. } => {
        to.assert_extends(&Type::Intrinsic { kind: Intrinsic::Void, span: self.get_span() })?;

        Ok(false)
      },
      Instruction::Call { .. } => todo!(),
      Instruction::Value(value) => value.coerce(checker, to),
    }
  }
}

impl Coerce for Value {
  fn coerce(&mut self, checker: &mut TypeChecker, to: &Type) -> Result<bool, TypeCheckerError> {
    match self {
      Value::Variable(var) => var.coerce(checker, to),
      Value::Instruction(inst) => inst.coerce(checker, to),
      Value::Literal { ty, .. } => ty.coerce(checker, to),
    }
  }
}
