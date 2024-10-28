use crate::compiler::error::{
  CompilerError,
  ReadSpan,
};

/// Interface for displaying errors once caught
pub(crate) trait LazyHelp: Sized {
  /// Whether this error causes the help options to be printed
  fn should_print_message(&self) -> bool {
    true
  }

  /// Whether this error is printed to the console
  fn should_print_help_text(&self) -> bool {
    false
  }

  // Returns a ReadSpan if one is applicable
  fn applicable_span(self) -> Option<ReadSpan> {
    None
  }
}

/// Shows help text
pub(super) fn print_help_text() {
  let full_executable_path = std::env::current_exe().unwrap();
  let executable = full_executable_path.file_name().unwrap().to_string_lossy();

  eprintln!("\
    Usage: {executable} [OPTION]... [INPUT]\n\
    \n\
    Options:\n  \
      -h, --help                             Shows this help message\n  \
      -i, --input=<FILE>                     Sets the program's entry file\n  \
      -o, --output=<FILE>                    Sets the program's output file\n  \
    \n\
    Tooling:\n  \
      --llc=<FILE>                           Path to the llc executable\n  \
      --cc=<FILE>                            Path to the cc executable\n  \
    \n\
    See LICENSE for more information.\
  ")
}

/// Prints a spanned error message
pub(super) fn print_message(err: CompilerError) {
  let message = err.to_string();

  let Some(span) = err.applicable_span() else {
    // If there's no span applicable here, then just print the message and move
    // along
    error!("{message}");
    return;
  };

  let header = format!("in {}:{}:{}", span.path.to_string_lossy(), span.line, span.column);

  // TODO: colorization, correct formatting ...
  error!("{message}\n{header}\n\n{}\n^ here", span.text);
}
