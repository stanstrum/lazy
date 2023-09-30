/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

mod aster;
mod typecheck;
mod codegen;

use getopts::{self, Options};

// const DEFAULT_OUTPUT: &str = "a.out";

use snafu::prelude::*;
use crate::aster::formatting::*;

use typecheck::errors::TypeCheckError;

pub(crate) mod colors;

use inkwell::context::Context;
use inkwell::builder::Builder;
use inkwell::module::Module;

use codegen::Codegen;

use std::process::Command;

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

  let reader = &mut aster::SourceReader::new(&src);

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
  println!("{};", asterized.to_string());

  // sponge: insert here an algorithm to rearrange operators by precedence

  let checked = {
    match typecheck::check(asterized) {
      Ok(checked) => checked,
      Err(err) => {
        match &err {
          TypeCheckError::NotImplemented { span, .. }
          | TypeCheckError::UnknownIdent { span, .. }
          | TypeCheckError::InvalidDot { span }
          | TypeCheckError::InvalidType { span, .. } => {
            let message = Message {
              level: Level::Error,
              msg: err.to_string(),
              sub: "here".to_owned(),
              span: span.to_owned(),
            };

            println!("{}", format_message(reader.src(), message));
          },
          TypeCheckError::DuplicateIdent { text, a, b } => {
            let message_a = Message {
              level: Level::Error,
              msg: err.to_string(),
              sub: "here".to_owned(),
              span: a.to_owned()
            };

            let message_b = Message {
              level: Level::Error,
              msg: err.to_string(),
              sub: "here".to_owned(),
              span: b.to_owned(),
            };

            println!("{}\n...{}",
              format_message(reader.src(), message_a),
              format_message(reader.src(), message_b)
            );
          }
        };

        return CompilationSnafu {
          msg: format!("Type check failed: {err}")
        }.fail();
      }
    }
  };

  dbg!(&checked);
  println!("{};", checked.to_string());

  // generate llvm
  let context = Context::create();
  let module = context.create_module("program");
  let builder = context.create_builder();

  let mut codegen = Codegen {
    context: &context,
    module: &module,
    builder: &builder
  };

  // codegen.init(todo!());

  if let Err(err) = codegen.generate_namespace(&checked) {
    return CompilationSnafu {
      msg: format!("Code generation failed: {err}")
    }.fail();
  };

  // write to file
  let cwd = std::env::current_dir().expect("couldn't get working dir");
  let out_file = cwd.join("a.ll");

  codegen
    .module
    .print_to_file(out_file.to_str().unwrap())
    .expect("error printing to file");

  // compile llvm code
  Command::new("llc")
    // this argument is surprisingly important
    .arg("--relocation-model=pic")
    .args(["-o a.s"])
    .arg("a.ll")
    .stdout(std::process::Stdio::piped())
    .spawn().unwrap()
    .wait()
    .expect("error compiling emitted llvm code");

  // assemble `llc` output
  Command::new("as")
    .args(["-o a.o"])
    .arg("a.s")
    .stdout(std::process::Stdio::piped())
    .spawn().unwrap()
    .wait()
    .expect("error assembling emitted assembly code");

  // link `as` output
  Command::new("cc")
    .args(["-o a.out"])
    .arg("a.o")
    .stdout(std::process::Stdio::piped())
    .spawn().unwrap()
    .wait()
    .expect("error linking emitted object code");

  Ok(())
}

fn print_usage(program: &str, opts: Options) {
  let brief = format!("Usage: {} [options] FILE", program);
  print!("{}", opts.usage(&brief));
}

fn main() {
  match compile() {
    Ok(_) | Err(LazyError::BadArguments) => {},
    Err(err) => {
      eprintln!("{}", err);
      std::process::exit(1);
    }
  };
}
