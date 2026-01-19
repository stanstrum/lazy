use std::fmt::Display;

use crate::lang::module::ModuleId;
use crate::tokenize::token::Span;
use crate::string_pool::PoolId;

#[derive(Debug)]
pub struct Qualified {
  pub parts: Vec<PoolId>,
  pub span: Span,
}

#[derive(Debug)]
pub enum Intrinsic {
  Void,
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

#[derive(Debug)]
pub enum Type {
  Unresolved {
    module: ModuleId,
    qualified: Qualified,
  },
  Intrinsic {
    kind: Intrinsic,
    span: Span,
  },
}

impl Intrinsic {
  fn try_from_str(str: &str) -> Option<Self> {
    match str {
      "void" => Some(Self::Void),
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
