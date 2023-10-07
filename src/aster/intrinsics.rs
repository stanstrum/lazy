/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use super::ast::{Type, QualifiedAST};
use phf::phf_ordered_map;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, PartialEq)]
pub enum Intrinsic {
  VOID,
  BOOL,
  CHAR,
  U8,
  U16,
  U32,
  U64,
  USIZE,
  I8,
  I16,
  I32,
  I64,
  ISIZE,
}

pub use Intrinsic::*;

pub static INTRINSICS_MAP: phf::OrderedMap<&'static str, Intrinsic> = phf_ordered_map! {
  "void" => VOID,
  "bool" => BOOL,
  "char" => CHAR,
  "u8" => U8,
  "u16" => U16,
  "u32" => U32,
  "u64" => U64,
  "usize" => USIZE,
  "i8" => I8,
  "i16" => I16,
  "i32" => I32,
  "i64" => I64,
  "isize" => ISIZE,
};

pub fn get_intrinsic(qual: &QualifiedAST) -> Option<Type> {
  if qual.parts.len() != 1 {
    return None;
  };

  let ident = qual.parts.first().unwrap();

  for (name, variant) in INTRINSICS_MAP.into_iter() {
    if &ident.text.as_str() == name {
      return Some(Type::Intrinsic(variant.to_owned()));
    };
  };

  None
}

impl Intrinsic {
  pub fn get_name(&self) -> String {
    for (name, variant) in INTRINSICS_MAP.into_iter() {
      if self == variant {
        return name.to_string();
      };
    };

    unreachable!("unknown intrinsic type {self:#?}")
  }
}
