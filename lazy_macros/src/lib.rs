#[macro_export]
macro_rules! colorize {
  ($color:expr) => {
    concat!("\x1b[", stringify!($color), "m")
  };
}
