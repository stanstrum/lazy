mod error;

mod lang;
mod domain;
mod preprocess;
mod check;

use std::collections::HashMap;

use crate::compiler::{
  // Compiler,
  Handle,
  SourceFile,
  SourceFileData,
};

use crate::tokenizer::Span;
use crate::CompilationError;

use check::TypeChecker;
pub(crate) use error::*;

use lang::VariableReference;
use preprocess::Preprocess;
use domain::*;

pub(crate) use domain::{
  Domain,
  Program,
};

#[allow(unused)]
pub(crate) struct Preprocessor {
  reference: DomainReference,
  modules: Program,
  scope_stack: Vec<HashMap<String, VariableReference>>,
}

impl Preprocessor {
  pub(crate) fn new(handle: Handle) -> Self {
    Self {
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

  fn find_variable_by_name(&self, name: &str, span: Span) -> Result<VariableReference, TypeCheckerError> {
    for scope in self.scope_stack.iter().rev() {
      if let Some(reference) = scope.get(name) {
        return Ok(reference.to_owned());
      };
    };

    UnknownVariableSnafu { name, span }.fail()
  }

  pub(crate) fn check(self, mut program: Program) -> Result<(), TypeCheckerError> {
    dbg!(&program);

    let mut checker = TypeChecker::new(&program);

    while checker.check(&mut program)? {};

    todo!()
  }
}
