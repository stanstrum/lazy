pub(crate) mod error;
mod parser;

use std::path::PathBuf;
use std::str::FromStr;

use error::*;
use parser::*;
use which::which;

use crate::compiler::error::*;
use crate::Result;

/// Parsed compiler options; paths have parsed & validated
#[allow(unused)]
#[derive(Debug)]
pub(crate) struct CompilerOptions {
  /// Whether to show help text and exit early
  pub(crate) help: bool,
  /// Program entry point
  pub(crate) input_file: Option<PathBuf>,
  /// Output location for executable
  pub(crate) output_file: PathBuf,
  /// Path to LLC executable
  pub(crate) llc: PathBuf,
  /// Path to CC executable
  pub(crate) cc: PathBuf,
  /// Print LLVM code during generation
  pub(super) print_llvm: bool,
}

/// Resolves a provided optional String into a path (with a provided default)
/// and maps the error into a CompilerError
fn default_option_resolve_path(path: Option<String>, default: &'static str) -> Result<PathBuf> {
  let path = path.as_deref().unwrap_or(default);

  match which(path) {
    Ok(x) => Ok(x),
    Err(err) => ExecNotFoundSnafu { path, err }.fail()?,
  }
}

/// Parses command-line arguments into CompilerOptions, or returns an error
pub(crate) fn parse() -> Result<CompilerOptions> {
  let mut parser = CompilerParser::new();

  for argument in std::env::args().skip(1) {
    parser.parse_argument(argument)?;
  }

  let input_file = if let Some(input_file) = &parser.input_file {
    let input_file = PathBuf::from_str(input_file).unwrap();

    match std::fs::canonicalize(&input_file) {
      Ok(x) => Some(x),
      Err(err) => {
        return IOSnafu {
          err: err.to_string(),
        }
        .fail()
      },
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
    print_llvm: parser.print_llvm,
  })
}
