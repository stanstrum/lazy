mod ty;
mod structure;
mod function;
mod expr;
mod reference;

use crate::line_dbg;

use crate::aster::pprint::Pretty;
use crate::lang::span::GetSpan;
use crate::lang::reference::{Reference, Store};
use crate::resolve::type_of::TypeOf;
use crate::resolve::{Coerce, Resolve};

use super::{Lazy, Result, Error, Tasks, tasks};
