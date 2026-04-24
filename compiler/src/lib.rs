mod lang;
mod tokenize;
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

use crate::lang::ModuleReference;
use crate::lang::module::Module;
use crate::lang::function::Function;
use crate::tokenize::token;

#[derive(Debug)]
pub struct Lazy<'pool> {
  pub(crate) pool: &'pool StringPool,
  pub(crate) pool_keys: keys::PoolKeys,
  pub(crate) settings: Settings,
  pub(crate) std: Option<ModuleReference>,
  pub(crate) modules: Vec<Module>,
  pub(crate) functions: Vec<Function>,
  pub(crate) tokens: Vec<Vec<token::TokenSpan>>,
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
