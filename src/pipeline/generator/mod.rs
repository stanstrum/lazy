mod impls;

use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use inkwell::context::Context;
use inkwell::values::FunctionValue;
use tempfile::NamedTempFile;

use crate::{Result, enchant};
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
    let module = compiler.context.create_module(self.handle.proper_name(compiler).as_str());

    input.borrow().generate(&mut self, &compiler.context)?;

    let llc_out = NamedTempFile::with_suffix(".s").expect("failed to make tmpfile").into_temp_path();
    let as_out = NamedTempFile::with_suffix(".o").expect("failed to make tmpfile").into_temp_path();

    {
      let llc_in = NamedTempFile::with_suffix(".ll").expect("failed to make tmpfile").into_temp_path();

      if compiler.settings.print_llvm {
        info!("{}: {}:\n{}",
          enchant!("--print-llvm"),
          self.handle.proper_name(compiler),
          module.print_to_string().to_string_lossy().trim()
        );
      };

      if let Err(err) = module.print_to_file(&llc_in) {
        return IOSnafu { err: err.to_string() }.fail()?;
      };

      trace!("{}: written LLVM to {}", enchant!("output"), llc_in.to_string_lossy());

      let mut command = Command::new(&compiler.settings.llc);

      command
        // this argument is surprisingly important
        .arg("--relocation-model=pic")
        .arg("-o")
        .arg(&llc_out)
        .arg(&llc_in)
        .stdout(std::io::stdout())
        .stderr(Stdio::piped());

      debug!("{} -c {command:?}", enchant!("sh"));

      let mut child = command.spawn().unwrap();
      let result = child.wait();

      let mut stderr_text = String::new();
      child.stderr.take().unwrap()
        .read_to_string(&mut stderr_text)
        .unwrap();

      let stderr_text = stderr_text.trim();
      if !stderr_text.is_empty() {
        error!("{}", stderr_text);
      };

      match result {
        Ok(x) if x.success() => {},
        Ok(x) => return IOSnafu { err: format!("cc returned {x}") }.fail()?,
        Err(err) => return IOSnafu { err: err.to_string() }.fail()?,
      };

      trace!("{} {llc_in:?}", enchant!("rm"));
    };

    {
      let mut command = Command::new(&compiler.settings.cc);

      command
        .stdout(std::io::stdout())
        .stderr(Stdio::piped())
        .arg("-c")
        .arg("-o")
        .arg(&as_out)
        .arg(&llc_out);

      debug!("{} -c {command:?}", enchant!("sh"));

      let mut child = command.spawn().unwrap();
      let result = child.wait();

      let mut stderr_text = String::new();
      child.stderr.take().unwrap()
        .read_to_string(&mut stderr_text)
        .unwrap();

      match result {
        Ok(x) if x.success() => {},
        Ok(x) => return IOSnafu { err: format!("cc returned {x}") }.fail()?,
        Err(err) => return IOSnafu { err: err.to_string() }.fail()?,
      };

      let stderr_text = stderr_text.trim();
      if !stderr_text.is_empty() {
        error!("{}", stderr_text);
      };
    };

    trace!("{} {llc_out:?}", enchant!("rm"));

    Ok(as_out.keep().unwrap())
  }
}
