mod lang;
mod tokenize;
mod aster;
mod resolve;
mod generate;

pub mod lazy;

pub mod error;
pub mod settings;

#[cfg(test)] mod test;

pub use string_pool::StringPool;
use crate::settings::Settings;

use crate::lazy::keys;
use crate::lang::reference::ModuleReference;
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
