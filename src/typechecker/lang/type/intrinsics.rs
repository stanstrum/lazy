pub(crate) const USIZE: Intrinsic = Intrinsic::U64;
pub(crate) const ISIZE: Intrinsic = Intrinsic::I64;
pub(crate) const UNICODE_CHAR: Intrinsic = Intrinsic::U32;
pub(crate) const C_CHAR: Intrinsic = Intrinsic::I8;

// pub(crate) const DEFAULT_UNSIGNED_INTEGER: Intrinsic = Intrinsic::U32;
// pub(crate) const DEFAULT_SIGNED_INTEGER: Intrinsic = Intrinsic::I32;

#[allow(unused)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Intrinsic {
  Void,
  U8,
  I8,
  U16,
  I16,
  U32,
  I32,
  U64,
  I64,
  F32,
  F64,
}

impl TryFrom<&str> for Intrinsic {
  type Error = ();

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "void" => Ok(Self::Void),
      "u8" => Ok(Self::U8),
      "i8" => Ok(Self::I8),
      "u16" => Ok(Self::U16),
      "i16" => Ok(Self::I16),
      "u32" => Ok(Self::U32),
      "i32" => Ok(Self::I32),
      "u64" => Ok(Self::U64),
      "i64" => Ok(Self::I64),
      "usize" => Ok(USIZE),
      "isize" => Ok(ISIZE),
      "f32" => Ok(Self::F32),
      "f64" => Ok(Self::F64),
      "char" => Ok(UNICODE_CHAR),
      _ => Err(()),
    }
  }
}
