use std::{fs::File, mem::take};

pub(self) mod tokenizer;
pub(self) mod asterizer;

pub(self) mod error;
mod debug;

use asterizer::AsterizerError;
pub(self) use error::CompilationError;
use error::*;
use tokenizer::{GetSpan, Token};

// TODO: make this more efficient -- this can be called multiple times per
//       error/warning/note, etc.
fn get_line_number(source: &String, index: usize) -> usize {
  // We humans start counting at one
  let mut line_number = 1;

  for ch in source.chars().take(index) {
    if ch == '\n' {
      line_number += 1;
    };
  };

  return line_number;
}

fn pretty_print_error(error: &AsterizerError, source: String) {
  let span = error.get_span();

  let mut focus_start = span.start;
  let mut focus_end = span.end;

  while focus_start > 0 && source.chars().nth(focus_start).unwrap() != '\n' {
    focus_start -= 1;
  };

  while focus_end < (source.len() - 1) && source.chars().nth(focus_end).unwrap() != '\n' {
    focus_end += 1;
  };

  focus_start += 1;
  focus_end -= 1;

  let focus_start_line_number = get_line_number(&source, focus_start);
  let focus_end_line_number = get_line_number(&source, focus_end);

  let line_number_max_digits = ((focus_end_line_number as f32).log10() + 1f32).floor() as usize;

  let empty_line_number = format!("{}", " ".repeat(line_number_max_digits));

  println!(" {empty_line_number} | Error: {error}", );
  println!(" {empty_line_number} |");

  let mut index = focus_start;
  for line_number in focus_start_line_number..=focus_end_line_number {
    print!(" {line_number: >line_number_max_digits$} | ");

    let mut did_newline = false;
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

      print!("{ch}");

      if ch == '\n' {
        did_newline = true;

        break;
      };
    };

    if !did_newline {
      println!();
    };

    let line_length = index - line_start;

    if should_do_squiggles {
      print!(" {empty_line_number} | ");

      let mut should_stop_squiggles = false;
      for col in (line_start..).take(line_length) {
        if col >= span.start && !should_stop_squiggles {
          if col == span.end {
            should_stop_squiggles = true;

            break;
          };

          print!("^");
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

fn compile(args: Vec<String>) -> Result<(), CompilationError> {
  let Some(input_file_path) = args.get(1) else {
    return ArgumentSnafu {
      message: "No input file provided."
    }.fail();
  };

  let input_file = match File::open(input_file_path) {
    Ok(file) => file,
    Err(error) => {
      return InputFileSnafu { error }.fail();
    }
  };

  let mut reader = utf8_read::Reader::new(input_file);

  let tokens = tokenizer::tokenize(&mut reader)?;
  let source = tokenizer::stringify(&tokens);

  // debug::tokens(&tokens);

  #[allow(unused_variables)]
  let ast = {
    match asterizer::asterize(tokens) {
      Ok(ast) => ast,
      Err(error) => {
        pretty_print_error(&error, source);

        return AsterizationSnafu { error }.fail();
      },
    }
  };

  // debug::ast(&ast);

  Ok(())
}

fn main() {
  let args: Vec<String> = std::env::args().collect();

  if let Err(error) = compile(args) {
    match &error {
      CompilationError::Argument { .. } => {
        eprintln!("Argument error: {error}");
      },
      CompilationError::InputFile { .. } => {
        eprintln!("Input file error: {error}");
      },
      CompilationError::Tokenization { .. } => {
        eprintln!("Tokenization error: {error}");
      },
      CompilationError::Asterization { .. } => {
        eprintln!("Asterization error: {error}");
      }
    };
  };
}

#[cfg(test)]
mod tests {
  use super::*;

  macro_rules! compile_snippet {
    ($test_name:ident, $source:literal) => {
      #[test]
      fn $test_name() {
        let args = vec![
          "test_suite",
          concat!(
            "snippets/",
            $source,
            ".zy"
          )
        ];

        let args = args
          .into_iter()
          .map(String::from)
          .collect();

        if let Err(err) = compile(args) {
          panic!("compilation failed: {err}");
        };
      }
    };
  }

  compile_snippet!(base_main, "00_base_main");
  compile_snippet!(assn, "01_assn");
  compile_snippet!(hello_world, "02_hello_world");
  // compile_snippet!(trait_imp, "03_trait_imp");
  compile_snippet!(extended_operators, "04_extended_operators");
  // compile_snippet!(counter_ns, "05_counter_ns");
  compile_snippet!(type_alias, "06_type_alias");
  // compile_snippet!(struct_stuff, "07_struct_stuff");
  compile_snippet!(codegen, "08_codegen");
  // compile_snippet!(r#extern, "09_extern");
  // compile_snippet!(read_source, "10_read_source");
  // compile_snippet!(import_std, "11_import_std");
  // compile_snippet!(structs, "12_structs");
  // compile_snippet!(struct_generic, "13_struct_generic");
  // compile_snippet!(slice, "14_slice");
  compile_snippet!(namespaces, "15_namespace");
  compile_snippet!(bare_bones, "bare_bones");
  // compile_snippet!(counter, "counter");
  // compile_snippet!(r#if, "if");
  compile_snippet!(message, "message");
  // compile_snippet!(namespaces, "namespaces");
  // compile_snippet!(std, "std");
  // compile_snippet!(string_ref, "string_ref");
  // compile_snippet!(type_make, "type_make");
}
