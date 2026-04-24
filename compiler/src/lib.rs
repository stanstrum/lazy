mod lang;
mod aster;
mod resolve;
mod generate;

pub mod lazy;
pub mod keys;

pub mod error;
pub mod settings;
pub mod format;

mod steps;

#[cfg(test)] mod test;

pub use string_pool::StringPool;
use crate::settings::Settings;

use crate::lang::{ModuleReference, TokenSpan};
use crate::lang::module::Module;
use crate::lang::function::Function;

pub mod tokenize {
  pub type Error = ::tokenize::Error<crate::lazy::LazyStructures>;
  pub type Tokenizer<'pool, const N: usize, T> = ::tokenize::Tokenizer<'pool, crate::lazy::LazyStructures, N, T>;
}

#[derive(Debug)]
pub struct Lazy<'pool> {
  pub(crate) pool: &'pool StringPool,
  pub(crate) pool_keys: keys::PoolKeys,
  pub(crate) settings: Settings,
  pub(crate) std: Option<ModuleReference>,
  pub(crate) modules: Vec<Module>,
  pub(crate) functions: Vec<Function>,
  pub(crate) tokens: Vec<Vec<TokenSpan>>,
}

impl<'pool> Lazy<'pool> {
  pub fn new(pool: &'pool StringPool, settings: Settings) -> Self {
    let lazy = Self {
      pool,
      settings,
      std: None,
      pool_keys: keys::PoolKeys::init(pool),
      modules: vec![],
      functions: vec![],
      tokens: vec![],
    };

    let argv = lazy.settings.argv.iter()
      .map(|arg| format::format_argument(arg))
      .collect::<Vec<_>>()
      .join(" ");

    print_message!(&lazy, {
      level: Debug,
      force: false,
      description: argv,
      contents: MessageContents::None,
    });

    lazy
  }
}
