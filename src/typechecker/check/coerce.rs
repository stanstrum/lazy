use crate::tokenizer::{
  Literal,
  LiteralKind,
  GetSpan,
};

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
  intrinsics::Intrinsic,
};

use crate::generator::ResolveToU32;

pub(in crate::typechecker) trait Coerce {
  fn coerce(&mut self, checker: &mut TypeChecker, to: &Type) -> Result<bool, TypeCheckerError>;
}

pub(in crate::typechecker) trait CoerceCell {
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
    let result = match (self, other) {
      (Type::Shared(lhs), rhs) => lhs.borrow().extends(rhs),
      (lhs, Type::Shared(rhs)) => lhs.extends(&*rhs.borrow()),
      (Type::Unknown { .. }, _) => dbg!(true),
      (Type::FuzzyString { size: lhs_size, element_ty: lhs_ty, .. }, Type::FuzzyString { size: rhs_size, element_ty: rhs_ty, .. }) => {
        if *lhs_size != *rhs_size {
          return dbg!(false);
        };

        if lhs_ty != rhs_ty {
          return dbg!(false);
        };

        dbg!(true)
      },
      (Type::FuzzyString { size, element_ty, span }, rhs) => {
        if !rhs.is_resolved() {
          return dbg!(false);
        };

        let span = span.to_owned();
        let element_ty: TypeCell = Type::Intrinsic {
          kind: element_ty.to_owned(),
          span,
        }.into();

        let sized_array_of = Type::SizedArrayOf {
          count: Value::Literal {
            literal: Literal {
              kind: LiteralKind::Integer(*size as u64),
              span,
            },
            ty: Type::FuzzyInteger { span }.into(),
          },
          ty: element_ty.to_owned(),
          span,
        };

        if rhs.extends(&sized_array_of) {
          return dbg!(true);
        };

        if rhs.extends(&Type::ReferenceTo {
          r#mut: false,
          ty: sized_array_of.into(),
          span,
        }) {
          return dbg!(true);
        };

        if rhs.extends(&Type::ReferenceTo {
          r#mut: false,
          ty: Type::UnsizedArrayOf {
            ty: element_ty.to_owned(),
            span,
          }.into(),
          span,
        }) {
          return dbg!(true);
        };

        dbg!(false)
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
      ) => dbg!(true),
      (Type::ReferenceTo {
        r#mut: mut_lhs,
        ty: ty_lhs,
        ..
      },
      Type::ReferenceTo {
        r#mut: mut_rhs,
        ty: ty_rhs,
        ..
      }) => {
        if *mut_lhs && !*mut_rhs {
          return dbg!(false);
        };

        ty_lhs.extends(ty_rhs)
      },
      (
        Type::SizedArrayOf {
          count: count_lhs,
          ty: ty_lhs,
          ..
        },
        Type::SizedArrayOf {
          count: count_rhs,
          ty: ty_rhs,
          ..
        },
      ) => {
        if count_lhs.resolve_to_u32() != count_rhs.resolve_to_u32() {
          return false;
        };

        ty_lhs.extends(ty_rhs)
      },
      (Type::UnsizedArrayOf { ty: ty_lhs, .. }, Type::UnsizedArrayOf { ty: ty_rhs, .. }) => {
        ty_lhs.extends(ty_rhs)
      },
      (Type::Intrinsic { kind: lhs, .. }, Type::Intrinsic { kind: rhs, .. }) => dbg!(lhs == rhs),
      _ => dbg!(false),
    };

    println!("{} == {} = {result}", self.pretty_print(), other.pretty_print());

    result
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

    let result = std::mem::discriminant(inner) != std::mem::discriminant(to);

    *inner = to.to_owned();

    Ok(result)
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
