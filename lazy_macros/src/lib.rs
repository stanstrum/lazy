#[macro_export]
macro_rules! colorize {
  ($color:expr) => {
    concat!("\x1b[", stringify!($color), "m")
  };
}

#[macro_export]
macro_rules! line_dbg {
  () => {
    line_dbg!("")
  };

  ($str:expr) => {
    concat!("[\x1b[1;4m", file!(), ":", line!(), "\x1b[0;24m]: ", $str)
  };
}

#[macro_export]
macro_rules! print_message {
  ($lazy:expr, $x:tt) => {
    #[allow(unused_imports)]
    use ::log::{
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
