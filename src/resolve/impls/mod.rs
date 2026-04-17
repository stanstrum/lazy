pub mod structure;
pub mod function;
pub mod ty;
mod expr;

mod tasks;

use crate::line_dbg;

use crate::aster::pprint::*;
use crate::lang::span::GetSpan;
use crate::lang::reference::{Reference, Store};
use crate::resolve::{Coerce, Resolve, TypeOf};

use super::{Lazy, Result, ErrorBase, Tasks};

#[macro_export]
macro_rules! print_message {
  ($lazy:expr, $x:tt) => {
    #[allow(unused_imports)]
    use $crate::error::{
      Level::*,
      *
    };
    print_message($lazy, PrintableMessage $x);
  }
}

#[macro_export]
macro_rules! print_once_per_thread {
  ($lazy:expr, $x:tt) => {
    let should_print = unsafe {
      static mut DID_RUN: bool = false;

      let should_print = !DID_RUN;
      DID_RUN = true;

      should_print
    };

    if should_print {
      use $crate::print_message;
      print_message!($lazy, $x);
    };
  };
}
