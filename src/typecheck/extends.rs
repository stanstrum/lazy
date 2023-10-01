/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

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
      (Type::Intrinsic(a), Type::Intrinsic(b)) => {
        let a = unsafe { &**a };
        let b = unsafe { &**b };

        a.name == b.name
      }
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
