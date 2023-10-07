/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::aster::intrinsics;

use super::*;

pub fn extends(ty: &Type, base: &Type) -> bool {
  println!("test: {:?} extends {:?}", ty, base);

  let result = {
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
      _ => {
        false
      }
    }
  };

  if result {
    println!("does extend");
  } else {
    println!("doesn't extend");
  };

  result
}
