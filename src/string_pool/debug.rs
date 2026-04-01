use super::{StringPool, PoolId};

impl StringPool {
  pub fn unravel_nodes(&self) -> impl Iterator<Item = String> {
    // find all the tails -- we manage this in the `insert` function, so even
    // substrings can be identified as distinct from their greater parts
    let tails = self.nodes.borrow()
      .iter()
      .enumerate()
      .filter_map(|(id, node)| {
        if node.tail {
          Some(PoolId(id))
        } else {
          None
        }
      }).collect::<Vec<_>>();

    // collect the strings so we can display them as if they weren't completely
    // illegible in the debug format
    tails.into_iter()
      .map(|id| self.get(id).collect())
  }
}

impl std::fmt::Debug for StringPool {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let strings = self.unravel_nodes().collect::<Vec<_>>();

    f.debug_struct("StringPool")
      .field("strings", &strings)
      .finish()
  }
}
