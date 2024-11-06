use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use tempfile::TempPath;

use super::*;

use crate::compiler::error::IOSnafu;
use crate::compiler::CompilerJob;
use crate::{Result, ok};
use crate::compiler::{
  CompilationStage,
  Compiler,
  CompilerStoreHandle,
  CompilerWorkflow,
  Output,
  workflow::DefaultWorkflow,
};

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Outputter<W: CompilerWorkflow> {
  handle: CompilerStoreHandle<W>,
  input: PathBuf,
}

impl Output<DefaultWorkflow> for Outputter<DefaultWorkflow> {
  type In = PathBuf;

  fn new(input: Self::In, handle: CompilerStoreHandle<DefaultWorkflow>) -> Self {
    Self {
      handle,
      input,
    }
  }

  fn output(self, compiler: &mut Compiler<DefaultWorkflow>) -> Result {
    let mut object_files = vec![self.input.to_path_buf()];

    for index in 0..compiler.store.modules.len() {
      let handle = CompilerStoreHandle::<DefaultWorkflow>::new(index);

      compiler.bring_to_stage(&handle, CompilationStage::Output)?;

      let object_file = match compiler.store.take_module(&handle).data {
        CompilerJob::Generated(x) => x,
        _ => continue,
      };

      object_files.push(object_file.to_path_buf());
    };

    let mut command = Command::new(&compiler.settings.cc);

    command
      .stdout(std::io::stdout())
      .stderr(Stdio::piped())
      .arg("-o")
      .arg(&compiler.settings.output_file)
      .args(&object_files);

    debug!("sh -c {command:?}");

    let mut child = command.spawn().unwrap();
    let result = child.wait();

    for object_file in object_files.into_iter() {
      trace!("rm {object_file:?}");
      TempPath::from_path(object_file);
    };

    let mut stderr_text = String::new();
    child.stderr.take().unwrap()
      .read_to_string(&mut stderr_text)
      .unwrap();

    if !stderr_text.is_empty() {
      error!("{}", stderr_text.trim());
    };

    match result {
      Ok(x) if x.success() => {},
      Ok(x) => return IOSnafu { err: format!("cc returned {x}") }.fail()?,
      Err(err) => return IOSnafu { err: err.to_string() }.fail()?,
    };

    info!("Your shiny new Lazy program is located in {}", compiler.settings.output_file.to_string_lossy());

    ok
  }
}
