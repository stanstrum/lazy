/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::{
  aster::intrinsics,
  codegen::parse_int_literal
};

use super::*;

pub fn assignable(a: &Type, b: &Type) -> bool {
  match (a, b) {
    (Type::Defined(a), _) => {
      let a = unsafe { &(**a).e };

      assignable(a, b)
    },
    (_, Type::Defined(b)) => {
      let b = unsafe { &(**b).e };

      assignable(a, b)
    },
    (Type::ConstReferenceTo(_), Type::ConstReferenceTo(_)) => {
      // todo: this may not work with LLVM & may cause segfaults
      // later on... revisit this.
      true
    },
    (Type::Intrinsic(a), Type::Intrinsic(b)) => {
      a == b
    },
    (Type::Struct(_, members_a), Type::Struct(_, members_b)) => {
      if members_a.len() != members_b.len() {
        return false;
      };

      for ((ty_a, _), (ty_b, _)) in members_a.iter().zip(members_b.iter()) {
        if !extends(ty_a, ty_b) {
          return false
        };
      };

      true
    },
    _ => false
  }
}

pub fn extends(ty: &Type, base: &Type) -> bool {
  match (ty, base) {
    (Type::Defined(ty), base) => {
      let ast = unsafe { &**ty };

      extends(&ast.e, base)
    },
    (ty, Type::Defined(base)) => {
      let ast = unsafe { &**base };

      extends(ty, &ast.e)
    },
    (Type::Struct(_, members_a), Type::Struct(_, members_b)) => {
      if members_a.len() != members_b.len() {
        return false;
      };

      for ((ty_a, _), (ty_b, _)) in members_a.iter().zip(members_b.iter()) {
        if !extends(ty_a, ty_b) {
          return false
        };
      };

      true
    },
    (Type::ConstReferenceTo(a), Type::ConstReferenceTo(b)) => {
      extends(&a.e, &b.e)
    },
    (Type::ArrayOf(a_len, a), Type::ArrayOf(b_len, b)) => {
      if let Some(a_len) = a_len {
        let Literal::IntLiteral(a_len) = &a_len.l else { unreachable!(); };

        if let Some(b_len) = b_len {
          let Literal::IntLiteral(b_len) = &b_len.l else { unreachable!(); };

          let a_len_parsed = parse_int_literal(a_len);
          let b_len_parsed = parse_int_literal(b_len);

          if a_len_parsed != b_len_parsed {
            return false;
          };
        };
      };

      extends(&a.e, &b.e)
    },
    // U8,
    // U16,
    // U32,
    // U64,
    // USIZE,
    // I8,
    // I16,
    // I32,
    // I64,
    // ISIZE,
    (
      Type::Intrinsic(
        | intrinsics::ISIZE
        | intrinsics::I64
        | intrinsics::USIZE
        | intrinsics::U64
      ),
      Type::Intrinsic(
        intrinsics::ISIZE
        | intrinsics::I64
        | intrinsics::USIZE
        | intrinsics::U64
      )
    ) => true,
    (
      Type::Intrinsic(intrinsics::CHAR | intrinsics::I32 | intrinsics::U32),
      Type::Intrinsic(intrinsics::CHAR | intrinsics::I32 | intrinsics::U32)
    ) => true,
    (Type::Intrinsic(intrinsics::ISIZE), Type::Intrinsic(intrinsics::U8))
    | (Type::Intrinsic(intrinsics::USIZE), Type::Intrinsic(intrinsics::U8))
    | (Type::Intrinsic(intrinsics::I64), Type::Intrinsic(intrinsics::U8))
    | (Type::Intrinsic(intrinsics::U64), Type::Intrinsic(intrinsics::U8))
    | (Type::Intrinsic(intrinsics::I32), Type::Intrinsic(intrinsics::U8))
    | (Type::Intrinsic(intrinsics::U32), Type::Intrinsic(intrinsics::U8))
    | (Type::Intrinsic(intrinsics::I16), Type::Intrinsic(intrinsics::U8))
    | (Type::Intrinsic(intrinsics::U16), Type::Intrinsic(intrinsics::U8))
    | (Type::Intrinsic(intrinsics::I8), Type::Intrinsic(intrinsics::U8)) => true,
    (Type::Intrinsic(a), Type::Intrinsic(b)) => {
      a == b
    },
    (_, Type::Generic(_, constraints)) => {
      for constraint in constraints.iter() {
        if !extends_constraint(ty, constraint) {
          return false;
        };
      };

      true
    },
    _ => {
      false
    }
  }
}

fn extends_constraint(_ty: &Type, constraint: &GenericConstraint) -> bool {
  match constraint {
    GenericConstraint::ExtendsTrait(_) => todo!("extends constraint"),
  }
}
