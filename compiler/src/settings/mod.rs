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
