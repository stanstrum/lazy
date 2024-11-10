mod impls;

use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use inkwell::context::Context;
use inkwell::values::FunctionValue;
use tempfile::NamedTempFile;

use crate::Result;
use crate::compiler::{
  Compiler,
  CompilerStoreHandle,
  CompilerWorkflow,
  Generate,
  workflow::DefaultWorkflow,
  error::*,
};

use crate::translator::lang::{RcCell, Module};

use super::translator::lang::ModuleChild;

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

trait LlvmGenerate<W: CompilerWorkflow> {
  type Out;

  fn generate_in_context(&self, context: &Context) -> Result<Self::Out>;
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
    let module = compiler.context.create_module(format!("{:?}", &input.borrow().name).as_str());

    input.borrow().generate(&mut self, &compiler.context)?;

    let llc_out = NamedTempFile::with_suffix(".s").expect("failed to make tmpfile").into_temp_path();
    let as_out = NamedTempFile::with_suffix(".o").expect("failed to make tmpfile").into_temp_path();

    {
      let llc_in = NamedTempFile::with_suffix(".ll").expect("failed to make tmpfile").into_temp_path();

      if compiler.settings.print_llvm {
        let path = &compiler.store.get_module(&self.handle).path;

        info!("output: module #{:?} {path}: llvm\n{}", &self.handle, module.print_to_string().to_string_lossy());
      };

      if let Err(err) = module.print_to_file(&llc_in) {
        return IOSnafu { err: err.to_string() }.fail()?;
      };

      trace!("output: written LLVM to {}", llc_in.to_string_lossy());

      let mut command = Command::new(&compiler.settings.llc);

      command
        // this argument is surprisingly important
        .arg("--relocation-model=pic")
        .arg("-o")
        .arg(&llc_out)
        .arg(&llc_in)
        .stdout(std::io::stdout())
        .stderr(Stdio::piped());

      debug!("sh -c {command:?}");

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

      trace!("rm {llc_in:?}");
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

      debug!("sh -c {command:?}");

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

    trace!("rm {llc_out:?}");

    Ok(as_out.keep().unwrap())
  }
}
