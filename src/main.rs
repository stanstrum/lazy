/* Copyright (c) 2024 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::fs::File;

pub(self) mod tokenizer;
pub(self) mod asterizer;

pub(self) mod error;
mod debug;

pub(self) use error::CompilationError;
use error::*;

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
  debug::tokens(&tokens);

  let ast = asterizer::asterize(tokens)?;

  debug::ast(&ast);

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
