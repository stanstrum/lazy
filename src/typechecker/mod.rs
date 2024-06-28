mod error;

mod lang;
mod domain;
mod preprocess;

use crate::compiler::{
  // Compiler,
  Handle,
  SourceFile,
  SourceFileData,
};

use crate::CompilationError;

pub(crate) use error::*;

use preprocess::Preprocess;
use domain::*;

pub(crate) use domain::Domain;

#[allow(unused)]
pub(crate) struct TypeChecker/* <'a> */ {
  // compiler: &'a Compiler,
  reference: DomainReference,
  modules: Program,
}

impl/* <'a> */ TypeChecker/* <'a> */ {
  pub(crate) fn new(/* compiler: &'a Compiler */ handle: Handle) -> Self {
    Self {
      // compiler,
      reference: DomainReference::new(handle),
      modules: Program::new(),
    }
  }

  pub(crate) fn preprocess(&mut self, file: SourceFile, handle: &Handle) -> Result<SourceFile, CompilationError> {
    let SourceFile {
      path,
      data: SourceFileData::Asterized(ast),
      debug_info,
    } = file else {
      unreachable!();
    };

    let program_reference = DomainReference::new(*handle);

    let program = dbg!(
      ast.preprocess(&program_reference)
    );

    Ok(SourceFile {
      path,
      data: SourceFileData::TypeChecked(program),
      debug_info,
    })
  }

  pub(crate) fn check(self) -> Result<(), TypeCheckerError> {
    todo!()
  }
}
