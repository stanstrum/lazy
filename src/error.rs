use std::path::Path;

use snafu::prelude::*;

use crate::asterizer::AsterizerError;
use crate::tokenizer::{
  TokenizationError,
  GetSpan
};
use crate::typechecker::TypeCheckerError;

use crate::colors::Color;

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

  #[snafu(display("{error}"))]
  TypeCheck { error: TypeCheckerError },
}

// TODO: make this more efficient -- this can be called multiple times per
//       error/warning/note, etc.
fn get_line_number(source: &str, index: usize) -> usize {
  // We "humans" start counting at one
  let mut line_number = 1;

  for ch in source.chars().take(index) {
    if ch == '\n' {
      line_number += 1;
    };
  };

  line_number
}

pub(super) fn pretty_print_error<T>(error: &T, source: &str, mut color_stream: Vec<(usize, Color)>, path: &Path)
  where T: GetSpan + std::fmt::Display
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

  if source.chars().nth(focus_start).unwrap() == '\n' {
    focus_start += 1;
  };

  if focus_end >= source.len() {
    focus_end = source.len() - 1;
  };

  while matches!(source.chars().nth(focus_end).unwrap(), '\n' | '\t' | ' ') {
    focus_end -= 1;
  };

  let focus_start_line_number = get_line_number(source, focus_start);
  let focus_end_line_number = get_line_number(source, focus_end);

  let line_number_max_digits = ((focus_end_line_number as f32).log10() + 1f32).floor() as usize;

  let empty_line_number = " ".repeat(line_number_max_digits);

  let divider = format!("{}|{}", Color::Creme.to_string(), Color::Clear.to_string());

  println!(" {empty_line_number} {divider} {}error{}: {error}", Color::Red.to_string(), Color::Clear.to_string());
  println!(" {empty_line_number} {divider}    {}in{}: {}", Color::Red.to_string(), Color::Clear.to_string(), path.to_string_lossy());
  println!(" {empty_line_number} {divider}");

  let mut index = focus_start;
  for line_number in focus_start_line_number..=focus_end_line_number {
    print!(" {}{line_number: >line_number_max_digits$}{} {divider} ", Color::Creme.to_string(), Color::Clear.to_string());

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

      // TODO: This feels very sloppy -- we have to do this because
      //       there may be colors in the stream that occur *before*
      //       the focus source appears, meaning we have to get rid of
      //       those first using the while loop.  Unfortunately this
      //       implementation also prints out extraneous control codes
      //       as a result.
      while let Some((color_start, color)) = color_stream.first() {
        if index <= *color_start {
          break;
        };

        print!("{}{}", Color::Clear.to_string(), color.to_string());

        color_stream.remove(0);
      };

      print!("{ch}");
    };

    println!("{}", Color::Clear.to_string());

    let line_length = index - line_start;

    if should_do_squiggles {
      print!(" {empty_line_number} {divider} {}", Color::LightGrey.to_string());

      let mut has_printed_squiggle = false;
      let mut should_stop_squiggles = false;
      for col in (line_start..).take(line_length) {
        if col >= span.start && !should_stop_squiggles {
          if col == span.end {
            should_stop_squiggles = true;

            if !has_printed_squiggle {
              print!("^");
            };

            break;
          };

          print!("^");
          has_printed_squiggle = true;
        } else {
          print!(" ");
        };
      };

      if should_stop_squiggles {
        print!(" here");
      };

      println!("{}", Color::Clear.to_string());
    };
  };

  println!(" {empty_line_number} {divider}");
}
