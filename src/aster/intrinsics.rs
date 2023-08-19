/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use super::ast::{IntrinsicType, Type};

static INTRINSICS: &[IntrinsicType] = &[
  IntrinsicType { name: "void", bytes: 0 },
  IntrinsicType { name: "bool", bytes: 1 },
  IntrinsicType { name: "char", bytes: 1 },
  IntrinsicType { name: "str", bytes: 0 },
  IntrinsicType { name: "u8", bytes: 1 },
  IntrinsicType { name: "u16", bytes: 2 },
  IntrinsicType { name: "u32", bytes: 4 },
  IntrinsicType { name: "u64", bytes: 8 },
  IntrinsicType { name: "usize", bytes: 4 },
  IntrinsicType { name: "i8", bytes: 1 },
  IntrinsicType { name: "i16", bytes: 2 },
  IntrinsicType { name: "i32", bytes: 4 },
  IntrinsicType { name: "i64", bytes: 8 },
  IntrinsicType { name: "isize", bytes: 4 },
];

pub fn get_intrinsic(name: &str) -> Option<Type> {
  for intrinsic in INTRINSICS.iter() {
    if intrinsic.name == name {
      return Some(Type::Intrinsic(intrinsic));
    }
  }

  None
}
