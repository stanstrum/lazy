mod lang;
mod resolve;
mod generate;

pub use ::structure::*;

pub mod error;
pub mod settings;

mod steps;

#[cfg(test)] mod test;

pub use string_pool::StringPool;
use crate::settings::Settings;

use crate::lang::{ModuleReference, TokenSpan};
use crate::lang::module::Module;
use crate::lang::function::Function;

pub use steps::*;

pub mod tokenize {
  pub type Error = ::tokenize::Error<crate::lazy::LazyStructures>;
  pub type Tokenizer<'pool, const N: usize, T> = ::tokenize::Tokenizer<'pool, crate::lazy::LazyStructures, N, T>;
}
