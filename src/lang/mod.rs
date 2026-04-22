pub mod reference;
pub mod module;
pub mod function;
pub mod ty;
pub mod expr;
pub mod span;
pub mod keys;

mod lazy;
pub use lazy::Lazy;

use std::path::{Path, PathBuf};

use crate::{line_dbg, print_message};
use crate::settings::Settings;
use crate::settings::format::format_argument;
use string_pool::StringPool;

use crate::lang::function::{Function, FunctionHeader};
use crate::lang::module::{Module, ModuleParent, ModulePath, TokensId};
use crate::lang::reference::{FunctionReference, ModuleReference, Store};
use crate::tokenize::token;

#[derive(Debug)]
pub enum LazyError {
  NotExist(PathBuf),
  Aster(crate::aster::Error),
}

impl From<crate::aster::Error> for LazyError {
  fn from(value: crate::aster::Error) -> Self {
    Self::Aster(value)
  }
}
