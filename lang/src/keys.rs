use string_pool::{PoolId, StringPool};

/// These are keys we need to have instantiated to avoid string comps and
/// such shenanigans later.  A little spaghetti but it makes sense for now.
/// Hoping that when the rewrite comes, this won't be necessary.
///
/// Could have these be lazy-evaluated but that's a little much comittal to the
/// whole 'lazy' joke, no?
#[derive(Debug)]
pub struct PoolKeys {
  pub super_: PoolId,
  pub void: PoolId,
  pub bool: PoolId,
  pub u8: PoolId,
  pub i8: PoolId,
  pub u16: PoolId,
  pub i16: PoolId,
  pub u32: PoolId,
  pub i32: PoolId,
  pub u64: PoolId,
  pub i64: PoolId,
  pub f32: PoolId,
  pub f64: PoolId,
}

impl PoolKeys {
  /// Initializes [`PoolKeys`] with the provided [`StringPool`].  See fields
  /// for special keywords cached here.
  pub fn init(pool: &StringPool) -> Self {
    Self {
      super_: pool.insert("super"),
      void: pool.insert("void"),
      bool: pool.insert("bool"),
      u8: pool.insert("u8"),
      i8: pool.insert("i8"),
      u16: pool.insert("u16"),
      i16: pool.insert("i16"),
      u32: pool.insert("u32"),
      i32: pool.insert("i32"),
      u64: pool.insert("u64"),
      i64: pool.insert("i64"),
      f32: pool.insert("f32"),
      f64: pool.insert("f64"),
    }
  }
}
