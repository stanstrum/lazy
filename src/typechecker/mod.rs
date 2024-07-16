mod error;
mod domain;
mod preprocess;
mod postprocess;
mod check;

pub(crate) mod lang;

use std::collections::HashMap;

use crate::compiler::{
  Handle,
  SourceFile,
  SourceFileData,
};

use crate::tokenizer::Span;

use check::TypeChecker;

use preprocess::Preprocess;

pub(crate) use check::TypeOf;
pub(crate) use domain::*;
pub(crate) use error::*;

#[allow(unused)]
pub(crate) struct Preprocessor {
  reference: DomainReference,
  modules: Program,
  scope_stack: Vec<HashMap<String, lang::VariableReference>>,
  template_scopes: Vec<Vec<(String, lang::Type)>>,
}

impl Preprocessor {
  pub(crate) fn new(handle: Handle) -> Self {
    Self {
      reference: DomainReference::new(handle),
      modules: Program::new(),
      scope_stack: vec![],
      template_scopes: vec![],
    }
  }

  pub(crate) fn preprocess(&mut self, file: SourceFile, _handle: &Handle) -> Result<SourceFile, TypeCheckerError> {
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

  fn find_variable_by_name(&self, name: &str, span: Span) -> Result<lang::VariableReference, TypeCheckerError> {
    for scope in self.scope_stack.iter().rev() {
      if let Some(reference) = scope.get(name) {
        return Ok(reference.to_owned());
      };
    };

    UnknownVariableSnafu { name, span }.fail()
  }

  pub(crate) fn check(self, program: &mut Program) -> Result<(), TypeCheckerError> {
    let mut checker = TypeChecker::new(program);

    checker.check_until_done(program)?;
    checker.postprocess(program)?;
    checker.check_until_done(program)
  }
}
