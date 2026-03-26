mod structure;
mod function;
mod reference;
mod ty;
mod expr;

mod tasks;

use crate::line_dbg;

use crate::aster::pprint::*;
use crate::lang::span::GetSpan;
use crate::lang::reference::{Reference, Store};
use crate::resolve::{Coerce, Resolve, TypeOf};

use super::{Lazy, Result, Error, Tasks};
