use snafu::prelude::*;

use crate::asterizer::AsterizerError;
use crate::tokenizer::{
  TokenizationError,
  GetSpan
};

#[derive(Snafu, Debug)]
#[snafu(visibility(pub(crate)))]
pub(crate) enum CompilationError {
  #[snafu(display("{message}"))]
  Argument { message: String },

  #[snafu(display("{error}"))]
  InputFile { error: std::io::Error },

  #[snafu(display("{error}"))]
  Tokenization { error: TokenizationError },

  #[snafu(display("{error}"))]
  Asterization { error: AsterizerError },
}

// TODO: make this more efficient -- this can be called multiple times per
//       error/warning/note, etc.
fn get_line_number(source: &str, index: usize) -> usize {
  // We humans start counting at one
  let mut line_number = 1;

  for ch in source.chars().take(index) {
    if ch == '\n' {
      line_number += 1;
    };
  };

  line_number
}

pub(super) fn pretty_print_error<'a, T>(error: &'a T, source: String)
  where T: GetSpan<'a> + std::fmt::Display
{
  let span = error.get_span();

  let mut focus_start = span.start;
  let mut focus_end = span.end;

  while focus_start > 0 && source.chars().nth(focus_start).unwrap() != '\n' {
    focus_start -= 1;
  };

  while focus_end < source.len() && source.chars().nth(focus_end).unwrap() != '\n' {
    focus_end += 1;
  };

  focus_start += 1;
  focus_end -= 1;

  let focus_start_line_number = get_line_number(&source, focus_start);
  let focus_end_line_number = get_line_number(&source, focus_end);

  let line_number_max_digits = ((focus_end_line_number as f32).log10() + 1f32).floor() as usize;

  let empty_line_number = " ".repeat(line_number_max_digits);

  println!(" {empty_line_number} | Error: {error}", );
  println!(" {empty_line_number} |");

  let mut index = focus_start;
  for line_number in focus_start_line_number..=focus_end_line_number {
    print!(" {line_number: >line_number_max_digits$} | ");

    let mut should_do_squiggles = false;

    let line_start = index;
    for ch in source[index..].chars() {
      if !should_do_squiggles && index >= span.start && index <= span.end {
        should_do_squiggles = true;
      };

      index += 1;

      if matches!(ch, '\r' /* or \v, \f, \0 */) {
        continue;
      };

      if ch == '\n' {
        break;
      };

      print!("{ch}");
    };

    println!();

    let line_length = index - line_start;

    if should_do_squiggles {
      print!(" {empty_line_number} | ");

      let mut should_stop_squiggles = false;
      for col in (line_start..).take(line_length) {
        if col >= span.start && !should_stop_squiggles {
          print!("^");

          if col == span.end {
            should_stop_squiggles = true;

            break;
          };
        } else {
          print!(" ");
        };
      };

      if should_stop_squiggles {
        print!(" here");
      };

      println!();
    };
  };

  println!(" {empty_line_number} |");
}
