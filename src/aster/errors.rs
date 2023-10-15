/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::path::PathBuf;

use snafu::prelude::*;

#[derive(Debug, Snafu, Clone)]
#[snafu(visibility(pub))]
pub enum AsterError {
  #[snafu(display("Expected {what}"))]
  Expected { what: String, offset: usize, path: PathBuf },

  #[snafu(display("Unknown {what}"))]
  Unknown { what: String, offset: usize, path: PathBuf },

  #[snafu(display("Bad literal: Expected {expected}"))]
  BadLiteral { expected: String, offset: usize, path: PathBuf },

  #[snafu(display("Couldn't open {}: {}", path.to_string_lossy().to_string(), error.to_string()))]
  ImportIO { error: String, offset: usize, path: PathBuf },

  #[snafu(display("NotImplemented {what}"))]
  NotImplemented { what: String, offset: usize, path: PathBuf },
}

impl AsterError {
  pub fn offset(&self) -> usize {
    match self {
      AsterError::Expected { offset, .. } => *offset,
      AsterError::Unknown { offset, .. } => *offset,
      AsterError::BadLiteral { offset, .. } => *offset,
      AsterError::ImportIO { offset, .. } => *offset,
      AsterError::NotImplemented { offset, .. } => *offset,
    }
  }

  pub fn src(&self) -> String {
    let path = match self {
      AsterError::Expected { path, .. } => path,
      AsterError::Unknown { path, .. } => path,
      AsterError::BadLiteral { path, .. } => path,
      AsterError::ImportIO { path, .. } => path,
      AsterError::NotImplemented { path, .. } => path,
    };

    std::fs::read_to_string(path)
      .expect("couldn't open file that was already opened")
  }
}

pub type AsterResult<T> = Result<T, AsterError>;
