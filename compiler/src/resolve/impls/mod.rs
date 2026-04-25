pub mod structure;
pub mod function;
pub mod ty;
mod expr;

mod tasks;

use crate::line_dbg;

use crate::aster::pprint::*;
use ::lang::span::GetSpan;
use crate::lang::{Reference, Store};
use crate::resolve::{Coerce, Resolve, TypeOf};

use super::{Lazy, Result, ErrorBase, Tasks};
