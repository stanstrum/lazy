  #[allow(unused)]
#[derive(Debug, Clone)]
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
      "u8" => Ok(Self::U8),
      "i8" => Ok(Self::I8),
      "u16" => Ok(Self::U16),
      "i16" => Ok(Self::I16),
      "u32" => Ok(Self::U32),
      "i32" => Ok(Self::I32),
      "u64" => Ok(Self::U64),
      "i64" => Ok(Self::I64),
      "f32" => Ok(Self::F32),
      "f64" => Ok(Self::F64),
      _ => Err(()),
    }
  }
}
