/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

mod aster;

use getopts::{self, Options};

// const DEFAULT_OUTPUT: &str = "a.out";

use snafu::prelude::*;
use crate::aster::formatting::*;

pub(crate) mod colors;

#[derive(Debug, Snafu)]
enum LazyError {
  BadArguments,

  #[snafu(display("IO Error: {}", msg))]
  IOError { msg: String },

  #[snafu(display("Compilation Error: {}", msg))]
  Compilation { msg: String },
}

fn compile() -> Result<(), LazyError> {
  let args: Vec<String> = std::env::args().collect();
  let program = args[0].clone();

  let mut opts = Options::new();
  opts.opt(
    "o",
    "out-file",
    "sets the output file name",
    "NAME",
    getopts::HasArg::No,
    getopts::Occur::Optional
  );
  opts.optflag("h", "help", "print this help menu");

  let matches = match opts.parse(&args[1..]) {
    Ok(m) => { m },
    Err(err) => { panic!("{}", err.to_string()) }
  };

  if matches.opt_present("h") {
    print_usage(&program, opts);

    return BadArgumentsSnafu.fail();
  }

  let input = if !matches.free.is_empty() {
    matches.free[0].clone()
  } else {
    println!("No input file specified.\n");

    print_usage(&program, opts);

    return BadArgumentsSnafu.fail();
  };

  println!("Input: {}", input);
  // println!("Output: {}", output);

  let mut path = std::env::current_dir()
    .expect("Unable to get working directory");

  path.push(input.clone());

  let src = match std::fs::read_to_string(&path) {
    Ok(src) => src,
    Err(err) => { return IOSnafu { msg: format!("{}: {}", path.to_string_lossy(), err) }.fail() }
  };

  let ref mut reader = aster::SourceReader::new(input, &src);

  let asterized = match aster::asterize(reader) {
    Ok(asterized) => asterized,
    Err(err) => {
      let message = Message {
        level: Level::Error,
        msg: err.to_string(),
        sub: "here".to_owned(),
        span: aster::Span {
          start: reader.offset(), end: reader.offset()
        }
      };

      return CompilationSnafu {
        msg: format_message(reader.src(), message)
      }.fail();
    }
  };

  dbg!(&asterized);
  println!("{}", asterized.to_string());

  // sponge: insert here an algorithm to rearrange operators by precedence

  println!("{}", asterized.to_string());

  todo!("code generation")
}

fn print_usage(program: &str, opts: Options) {
  let brief = format!("Usage: {} [options] FILE", program);
  print!("{}", opts.usage(&brief));
}

fn main() {
  let result = compile();

  match result {
    Ok(_) | Err(LazyError::BadArguments) => (),
    Err(ref err) => {
      eprintln!("{}", err);
    }
  };

  match result {
    Ok(_) => (),
    Err(_) => std::process::exit(1)
  }
}
