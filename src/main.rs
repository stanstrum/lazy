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

  let lexed = match aster::asterize(reader) {
    Ok(lexed) => lexed,
    Err(err) => {
      return CompilationSnafu {
        msg: format!(
          "{} at:\n{}",
          err.to_string(),
          reader.at()
        )
      }.fail()
    }
  };

  dbg!(&lexed);

  println!("{}",
    lexed.to_string()
  );

  todo!()
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
