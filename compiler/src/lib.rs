mod lang {
  pub use ::gluezy::prelude::*;
  pub use ::lang::reference::{Store, Reference};
}

mod resolve;
mod generate;

pub mod error;
pub mod settings;

mod steps;

#[cfg(test)] mod test;

pub use string_pool::StringPool;

pub use steps::*;

pub mod tokenize {
  pub type Error = ::tokenize::Error<gluezy::LazyStructures>;
  pub type Tokenizer<'pool, const N: usize, T> = ::tokenize::Tokenizer<'pool, gluezy::LazyStructures, N, T>;
}
