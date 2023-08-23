/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use super::ast::{IntrinsicType, Type, QualifiedAST};

pub const VOID: IntrinsicType = IntrinsicType { name: "void", bytes: 0 };
pub const BOOL: IntrinsicType = IntrinsicType { name: "bool", bytes: 1 };
pub const CHAR: IntrinsicType = IntrinsicType { name: "char", bytes: 1 };
pub const U8: IntrinsicType = IntrinsicType { name: "u8", bytes: 1 };
pub const U16: IntrinsicType = IntrinsicType { name: "u16", bytes: 2 };
pub const U32: IntrinsicType = IntrinsicType { name: "u32", bytes: 4 };
pub const U64: IntrinsicType = IntrinsicType { name: "u64", bytes: 8 };
pub const USIZE: IntrinsicType = IntrinsicType { name: "usize", bytes: 4 };
pub const I8: IntrinsicType = IntrinsicType { name: "i8", bytes: 1 };
pub const I16: IntrinsicType = IntrinsicType { name: "i16", bytes: 2 };
pub const I32: IntrinsicType = IntrinsicType { name: "i32", bytes: 4 };
pub const I64: IntrinsicType = IntrinsicType { name: "i64", bytes: 8 };
pub const ISIZE: IntrinsicType = IntrinsicType { name: "isize", bytes: 4 };

static INTRINSICS: &[IntrinsicType] = &[
  VOID, BOOL, CHAR,
  U8, U16, U32, U64, USIZE,
  I8, I16, I32, I64, ISIZE
];

pub fn get_intrinsic(qual: &QualifiedAST) -> Option<Type> {
  if qual.parts.len() != 1 {
    return None;
  };

  let ident = &qual.parts[0];

  for intrinsic in INTRINSICS.iter() {
    if intrinsic.name == ident.text {
      return Some(Type::Intrinsic(intrinsic));
    };
  };

  None
}
