mod error;

mod lang;
mod domain;
mod preprocess;

use std::collections::HashMap;

use crate::compiler::{
  // Compiler,
  Handle,
  SourceFile,
  SourceFileData,
};

use crate::CompilationError;

pub(crate) use error::*;

use lang::VariableReference;
use preprocess::Preprocess;
use domain::*;

pub(crate) use domain::{
  Domain,
  Program,
};

#[allow(unused)]
pub(crate) struct TypeChecker/* <'a> */ {
  // compiler: &'a Compiler,
  reference: DomainReference,
  modules: Program,
  scope_stack: Vec<HashMap<String, VariableReference>>,
}

impl/* <'a> */ TypeChecker/* <'a> */ {
  pub(crate) fn new(/* compiler: &'a Compiler */ handle: Handle) -> Self {
    Self {
      // compiler,
      reference: DomainReference::new(handle),
      modules: Program::new(),
      scope_stack: vec![],
    }
  }

  pub(crate) fn preprocess(&mut self, file: SourceFile, _handle: &Handle) -> Result<SourceFile, CompilationError> {
    let SourceFile {
      path,
      data: SourceFileData::Asterized(ast),
      debug_info,
    } = file else {
      unreachable!();
    };

    let domain = ast.preprocess(self)?;

    Ok(SourceFile {
      path,
      data: SourceFileData::TypeChecked(domain),
      debug_info,
    })
  }

  fn find_variable_by_name(&self, name: &str) -> Result<VariableReference, TypeCheckerError> {
    for scope in self.scope_stack.iter().rev() {
      if let Some(reference) = scope.get(name) {
        return Ok(reference.to_owned());
      };
    };

    UnknownVariableSnafu { name }.fail()
  }

  pub(crate) fn check(self, program: Program) -> Result<(), TypeCheckerError> {
    dbg!(&program);

    todo!()
  }
}
