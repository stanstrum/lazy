pub(crate) mod error;
mod parser;

use crate::Result;
use which::which;

use std::path::PathBuf;
use std::str::FromStr;

use crate::compiler::error::*;

use parser::*;
use error::*;

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct CompilerOptions {
  pub(crate) help: bool,
  pub(crate) input_file: Option<PathBuf>,
  pub(crate) output_file: PathBuf,
  pub(crate) llc: PathBuf,
  pub(crate) cc: PathBuf,
}

fn default_option_resolve_path(path: Option<String>, default: &'static str) -> Result<PathBuf>  {
  let path = path.as_ref()
    .map(|path| path.as_str())
    .unwrap_or(default);

  match which(path) {
    Ok(x) => Ok(x),
    Err(err) => ExecNotFoundSnafu { path, err }.fail()?
  }
}

pub(crate) fn parse() -> Result<CompilerOptions> {
  let mut parser = CompilerParser::new();

  for argument in std::env::args().skip(1) {
    parser.parse_argument(argument)?;
  };

  let input_file = if let Some(input_file) = &parser.input_file {
    let input_file = PathBuf::from_str(&input_file).unwrap();

    match std::fs::canonicalize(&input_file) {
      Ok(x) => Some(x),
      Err(err) => return IOSnafu { err: err.to_string() }.fail(),
    }
  } else {
    None
  };

  let output_file = parser.output_file.unwrap_or("a.out".into());
  let output_file = PathBuf::from_str(&output_file).unwrap();

  let llc = default_option_resolve_path(parser.llc, "llc")?;
  let cc = default_option_resolve_path(parser.cc, "cc")?;

  Ok(CompilerOptions {
    help: parser.help,
    input_file,
    output_file,
    llc,
    cc,
  })
}
