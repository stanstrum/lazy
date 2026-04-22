pub mod format;

use std::path::PathBuf;

use crate::error::Level;

#[derive(Debug)]
pub struct Settings {
  pub executable: String,
  pub input_path: PathBuf,
  pub output_path: PathBuf,
  pub log_level: Level,
  pub argv: Vec<String>,
}

#[derive(Debug)]
pub enum Verb {
  Check,
  Build,
  Run,
}

#[derive(Debug)]
pub enum Error {
  Missing {
    what: &'static str,
    position: usize,
  },
  Invalid {
    what: &'static str,
    position: usize,
  },
  Verbless,
  Version,
  Help,
}
