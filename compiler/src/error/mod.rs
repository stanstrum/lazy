mod seek;
mod print;

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::fs::File;

use lazy_macros::{colorize, line_dbg};
use gluezy::{Lazy, LazyStructures, ModuleReference};
use log::{Level, MessageContents, MessageSection, PrintableMessage, WithinSource};
use crate::lang::Span;

pub use print::print_message;
