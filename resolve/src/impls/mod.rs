pub mod structure;
pub mod function;
pub mod ty;
mod expr;

mod tasks;

use lazy_macros::line_dbg;

use ::pprint::*;
use ::lang::span::GetSpan;
use crate::lang::{Reference, Store};
use ::resolve::{Coerce, Resolve};
use ::lang::ty::TypeOf;

use super::{Result, ErrorBase, Tasks};
