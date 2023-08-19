use snafu::prelude::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum AsterError {
  #[snafu(display("Expected {what}"))]
  Expected { what: String, offset: usize },

  #[snafu(display("Unknown {what}"))]
  Unknown { what: String, offset: usize },

  #[snafu(display("NotImplemented {what}"))]
  NotImplemented { what: String, offset: usize },
}

pub type AsterResult<T> = Result<T, AsterError>;
