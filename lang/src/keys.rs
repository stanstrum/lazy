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
}

impl PoolKeys {
  /// Initializes [`PoolKeys`] with the provided [`StringPool`].  See fields
  /// for special keywords cached here.
  pub fn init(pool: &StringPool) -> Self {
    Self {
      super_: pool.insert("super"),
    }
  }
}
