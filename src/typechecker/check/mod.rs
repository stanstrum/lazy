use super::{
  Program,
  Preprocessor,
  TypeCheckerError,
};

pub(super) trait Check {
  fn check(&mut self, preprocessor: &Preprocessor) -> Result<bool, TypeCheckerError>;
}

impl Check for Program {
  fn check(&mut self, _preprocessor: &Preprocessor) -> Result<bool, TypeCheckerError> {
    todo!()
  }
}
