use crate::tokenizer::{GetSpan, Literal, LiteralKind};
use crate::typechecker::{
  TypeChecker,
  error::*,
};

use crate::typechecker::lang::{
  Instruction,
  VariableReference,
  Value,
  Type,
  TypeCell,
};

pub(super) trait Coerce {
  fn coerce(&mut self, checker: &mut TypeChecker, to: &Type) -> Result<bool, TypeCheckerError>;
}

pub(crate) trait Extends where Self: std::fmt::Debug + crate::typechecker::lang::pretty_print::PrettyPrint + GetSpan {
  fn extends(&self, other: &Self) -> bool;
  fn assert_extends(&self, other: &Self) -> Result<(), TypeCheckerError> {
    if self.extends(other) {
      Ok(())
    } else {
      IncompatibleTypesSnafu {
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

        if ty.extends(&Type::UnsizedArrayOf {
          ty: element_ty.to_owned(),
          span: span.to_owned(),
        }.into()) {
          return true;
        };

        // TODO: FIXME: bad bad not good
        if ty.extends(&Type::SizedArrayOf {
          count: Value::Instruction(
            Box::new(
              Instruction::Literal(
                Literal {
                  kind: LiteralKind::Integer(*size as u64),
                  span: span.to_owned(),
                }
              )
            )
          ),
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

    match (&inner, to) {
      (Type::Unknown { .. }, _) => {
        *inner = to.to_owned();

        Ok(true)
      },
      _ => panic!("not coercible:\n- {inner:#?}\n\n- {to:#?}")
    }
  }
}

impl Coerce for VariableReference {
  fn coerce(&mut self, checker: &mut TypeChecker, to: &Type) -> Result<bool, TypeCheckerError> {
    self.get().ty.coerce(checker, to)
  }
}

impl Coerce for Instruction {
  fn coerce(&mut self, _checker: &mut TypeChecker, _to: &Type) -> Result<bool, TypeCheckerError> {
    todo!()
  }
}

impl Coerce for Value {
  fn coerce(&mut self, checker: &mut TypeChecker, to: &Type) -> Result<bool, TypeCheckerError> {
    match self {
      Value::Variable(var) => var.coerce(checker, to),
      Value::Instruction(inst) => inst.coerce(checker, to),
    }
  }
}
