pub mod structure;
pub mod function;
pub mod ty;
mod expr;

mod tasks;

use lazy_macros::line_dbg;

use pprint::*;
use lang::span::GetSpan;
use lang::reference::{Reference, Store};
use lang::ty::TypeOf;
use crate::{Coerce, Resolve};

use super::{Result, ResolveErrorBase, Tasks};
