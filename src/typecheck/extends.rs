/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::aster::intrinsics;

use super::*;

pub fn assignable(a: &Type, b: &Type) -> bool {
  // println!("test: {:?} assignable to {:?}", a, b);

  // let result = {
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
      }
      _ => false
    }
  // };

  // if result {
  //   println!("does assign");
  // } else {
  //   println!("doesn't assign");
  // };

  // result
}

pub fn extends(ty: &Type, base: &Type) -> bool {
  // println!("test: {:?} extends {:?}", ty, base);

  // let result = {
    match (ty, base) {
      (Type::Defined(ty), base) => {
        let ast = unsafe { &**ty };

        extends(&ast.e, base)
      },
      (ty, Type::Defined(base)) => {
        let ast = unsafe { &**base };

        extends(ty, &ast.e)
      },
      (Type::Struct(a), Type::Struct(b)) => {
        a == b
      },
      (Type::ConstReferenceTo(a), Type::ConstReferenceTo(b)) => {
        extends(&a.e, &b.e)
      },
      (Type::ArrayOf(None, a), Type::ArrayOf(None, b)) => {
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
  // };

  // if result {
  //   println!("does extend");
  // } else {
  //   println!("doesn't extend");
  // };

  // result
}

fn extends_constraint(_ty: &Type, constraint: &GenericConstraint) -> bool {
  match constraint {
    GenericConstraint::ExtendsTrait(_) => todo!("extends constraint"),
  }
}
