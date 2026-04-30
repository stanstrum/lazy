use string_pool::PoolId;

use crate::keys::PoolKeys;

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

impl Intrinsic {
  pub fn is_integer(&self) -> bool {
    self.is_unsigned_integer() || self.is_signed_integer()
  }

  pub fn is_unsigned_integer(&self) -> bool {
    matches!(self,
      | Self::U8
      | Self::U16
      | Self::U32
      | Self::U64
    )
  }

  pub fn is_signed_integer(&self) -> bool {
    matches!(self,
      | Self::I8
      | Self::I16
      | Self::I32
      | Self::I64
    )
  }

  // pub fn is_floating_point(&self) -> bool {
  //   matches!(self,
  //     | Self::F32
  //     | Self::F64
  //   )
  // }

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

  // SPONGE: refactor this somehow
  pub fn try_from_keys(id: PoolId, keys: &PoolKeys) -> Option<Self> {
    if id == keys.void {
      return Some(Self::Void);
    };

    if id == keys.bool {
      return Some(Self::Bool);
    };

    if id == keys.u8 {
      return Some(Self::U8);
    };

    if id == keys.u16 {
      return Some(Self::U16);
    };

    if id == keys.u32 {
      return Some(Self::U32);
    };

    if id == keys.u64 {
      return Some(Self::U64);
    };

    if id == keys.i8 {
      return Some(Self::I8);
    };

    if id == keys.i16 {
      return Some(Self::I16);
    };

    if id == keys.i32 {
      return Some(Self::I32);
    };

    if id == keys.i64 {
      return Some(Self::I64);
    };

    if id == keys.f32 {
      return Some(Self::F32);
    };

    if id == keys.f64 {
      return Some(Self::F64);
    };

    None
  }
}

impl std::fmt::Display for Intrinsic {
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
