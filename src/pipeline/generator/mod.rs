mod impls;

use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use inkwell::context::Context;
use inkwell::values::FunctionValue;
use tempfile::NamedTempFile;

use crate::{Result, ok, enchant};
use crate::compiler::{
  Compiler,
  CompilerStoreHandle,
  CompilerWorkflow,
  Generate,
  workflow::DefaultWorkflow,
  error::*,
};

use crate::translator::lang::{RcCell, Module};

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Generator<W: CompilerWorkflow> {
  handle: CompilerStoreHandle<W>,
  input: Option<RcCell<Module>>,
  context: Context,
  functions: Vec<FunctionValue<'static>>,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct GeneratorModule {
  context: Context,
  module: inkwell::module::Module<'static>,
}

impl Generate<DefaultWorkflow> for Generator<DefaultWorkflow> {
  type In = RcCell<Module>;
  type Out = PathBuf;

  fn new(input: Self::In, handle: CompilerStoreHandle<DefaultWorkflow>) -> Self {
    Self {
      input: Some(input),
      handle,
      context: Context::create(),
      functions: vec![],
    }
  }

  fn generate(mut self, compiler: &mut Compiler<DefaultWorkflow>) -> Result<Self::Out> {
    let input = self.input.take().unwrap();
    let module = input.borrow().generate(&mut self, &compiler.context)?;

    let object_file = NamedTempFile::with_suffix(".o").expect("failed to make tmpfile").into_temp_path();

    if compiler.settings.print_llvm {
      info!("{}: {}:\n{}",
        enchant!("--print-llvm"),
        self.handle.proper_name(compiler),
        module.print_to_string().to_string_lossy().trim()
      );
    };

    let mut llc = Command::new(&compiler.settings.llc);
    let mut assembler = Command::new(&compiler.settings.cc);

    llc
      // this argument is surprisingly important
      .arg("--relocation-model=pic")
      .stdout(Stdio::piped())
      .stdin(Stdio::piped())
      .stderr(Stdio::piped());

    assembler
      .arg("-c")
      .args(["-x", "assembler"])
      .arg("-o")
      .arg(&object_file)
      .arg("-")
      .stdout(std::io::stdout())
      .stdin(Stdio::piped())
      .stderr(Stdio::piped());

    trace!("{} -c {:?}", enchant!("sh"), &llc);
    let mut llc_child = llc.spawn().expect("spawn llc subprocess");
    let mut llc_out = llc_child.stdout.take().unwrap();
    let mut llc_in = llc_child.stdin.take().unwrap();
    let mut llc_err = llc_child.stderr.take().unwrap();

    let bitcode = module.write_bitcode_to_memory();
    let bitcode = bitcode.as_slice().to_owned();

    let llc_writer = std::thread::spawn(move || {
      llc_in
        .write_all(&bitcode)
        .expect("failed to write bitcode to llc stdin");
    });

    let llc_error_reader = std::thread::spawn(move || {
      let mut stderr = String::new();

      llc_err
        .read_to_string(&mut stderr)
        .unwrap();

      let stderr = stderr.trim();
      if !stderr.is_empty() {
        error!("{stderr}");
      };
    });

    trace!("{} -c {:?}", enchant!("sh"), &assembler);
    let mut assembler_child = assembler.spawn().expect("spawn assembler subprocess");
    let mut assembler_in = assembler_child.stdin.take().unwrap();
    let mut assembler_err = assembler_child.stderr.take().unwrap();

    let assembler_pipe = std::thread::spawn(move || {
      let mut buffer = [0; 1024];

      loop {
        match llc_out.read(&mut buffer) {
          Ok(0) => break,
          Ok(size) => {
            if let Err(err) = assembler_in.write(&buffer[..size]) {
              return IOSnafu { err: err.to_string() }.fail()?;
            };
          },
          Err(err) => {
            return IOSnafu { err: err.to_string() }.fail()?;
          },
        };
      };

      ok
    });

    let assembler_error_reader = std::thread::spawn(move || {
      let mut stderr = String::new();
      assembler_err
        .read_to_string(&mut stderr)
        .unwrap();

      let stderr = stderr.trim();
      if !stderr.is_empty() {
        error!("{stderr}");
      };
    });

    // join pipe threads
    llc_writer.join().expect("couldn't join llc bitcode writer");
    llc_error_reader.join().expect("couldn't join assembler pipe");
    assembler_pipe.join().expect("couldn't join assembler pipe")?;
    assembler_error_reader.join().expect("couldn't join assembler pipe");

    match llc_child.wait() {
      Ok(x) if x.success() => {},
      Ok(x) => return IOSnafu { err: format!("llc returned {x}") }.fail()?,
      Err(err) => return IOSnafu { err: err.to_string() }.fail()?,
    };

    match assembler_child.wait() {
      Ok(x) if x.success() => {},
      Ok(x) => return IOSnafu { err: format!("llc returned {x}") }.fail()?,
      Err(err) => return IOSnafu { err: err.to_string() }.fail()?,
    };

    Ok(object_file.keep().unwrap())
  }
}
