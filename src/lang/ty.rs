use std::fmt::Display;

use crate::lang::module::{Name};
use crate::lang::reference::{ModuleReference, TypePartReference, TypeReference};
use crate::tokenize::token::Span;

#[derive(Debug, Clone)]
pub struct Qualified {
  pub implicit: bool,
  pub parts: Vec<Name>,
  pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(unused)]
pub enum Intrinsic {
  Void,
  Bool,
  U8,
  U16,
  U32,
  U64,
  I8,
  I16,
  I32,
  I64,
  F32,
  F64,
}

#[derive(Debug, Clone)]
pub enum Type {
  Reference(TypeReference),
  Unresolved {
    module: ModuleReference,
    qualified: Qualified,
  },
  Intrinsic {
    kind: Intrinsic,
    span: Span,
  },
  // Resolved {
  //   original: Box<Type>,
  //   reference: TypeReference,
  // },
  WeakInteger {
    span: Span,
  },
  WeakFloat {
    span: Span,
  },
  WeakString {
    span: Span,
  },
  Weak {
    span: Span,
  },
  ReferenceTo {
    ty: TypePartReference,
    r#mut: bool,
    span: Span,
  },
  UnsizedArrayOf {
    ty: TypePartReference,
    span: Span,
  },
  SizedArrayOf {
    ty: TypePartReference,
    size: u64,
    span: Span,
  },
  // Block(BlockReference),
}

impl Intrinsic {
  pub fn try_from_str(str: &str) -> Option<Self> {
    match str {
      "void" => Some(Self::Void),
      "bool" => Some(Self::Bool),
      "u8" => Some(Self::U8),
      "u16" => Some(Self::U16),
      "u32" => Some(Self::U32),
      "u64" => Some(Self::U64),
      "i8" => Some(Self::I8),
      "i16" => Some(Self::I16),
      "i32" => Some(Self::I32),
      "i64" => Some(Self::I64),
      "f32" => Some(Self::F32),
      "f64" => Some(Self::F64),
      _ => None,
    }
  }
}

impl Display for Intrinsic {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(match self {
      Intrinsic::Void => "void",
      Intrinsic::Bool => "bool",
      Intrinsic::U8 => "u8",
      Intrinsic::U16 => "u16",
      Intrinsic::U32 => "u32",
      Intrinsic::U64 => "u64",
      Intrinsic::I8 => "i8",
      Intrinsic::I16 => "i16",
      Intrinsic::I32 => "i32",
      Intrinsic::I64 => "i64",
      Intrinsic::F32 => "f32",
      Intrinsic::F64 => "f64",
    })
  }
}
