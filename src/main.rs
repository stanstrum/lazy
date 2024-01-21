/* Copyright (c) 2024 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::fs::File;
use std::io::BufReader;

use snafu::prelude::*;
use tokenizer::TokenizationError;

pub(self) mod tokenizer;

#[derive(Snafu, Debug)]
enum CompilationError {
  #[snafu(display("{message}"))]
  Argument { message: String },

  #[snafu(display("{error}"))]
  InputFile { error: std::io::Error },

  #[snafu(display("{error}"))]
  Tokenization { error: tokenizer::TokenizationError },
}

impl From<TokenizationError> for CompilationError {
  fn from(error: TokenizationError) -> Self {
    Self::Tokenization { error }
  }
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

  dbg!(&tokens);

  Ok(())
}

fn main() {
  let args: Vec<String> = std::env::args().collect();

  match compile(args) {
    Ok(_) => {},
    Err(error) => {
      eprintln!("Error: {error}");
    },
  };
}
