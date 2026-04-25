mod seek;
mod print;

use std::io::{BufRead, BufReader, Write};
use std::fs::File;

use lazy_macros::colorize;
use gluezy::{Lazy, LazyStructures, ModuleReference};
use log::{MessageContents, MessageSection, PrintableMessage, WithinSource};
use crate::lang::Span;

pub use print::print_message;
