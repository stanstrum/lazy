/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use snafu::prelude::*;

use crate::aster::Span;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum TypeCheckError {
  #[snafu(display("Not implemented: {text}"))]
  NotImplemented { text: String, span: Span },

  #[snafu(display("Unknown identifier: {text}"))]
  UnknownIdent { text: String, span: Span },

  #[snafu(display("Duplicate identifier: {text}"))]
  DuplicateIdent { text: String, a: Span, b: Span },

  #[snafu(display("Invalid type: {text}"))]
  InvalidType { text: String, span: Span },

  #[snafu(display("{what} is not compatible with {with}"))]
  IncompatibleType { span: Span, what: String, with: String },

  #[snafu(display("Failed to infer type"))]
  CantInferType { span: Span },

  #[snafu(display("Invalid Dot/DerefDot Operator"))]
  InvalidDot { span: Span }
}

impl TypeCheckError {
  pub fn src(&self) -> String {
    let path = match self {
        TypeCheckError::NotImplemented { span, .. } => &span.path,
        TypeCheckError::UnknownIdent { span, .. } => &span.path,
        TypeCheckError::DuplicateIdent { b, .. } => &b.path,
        TypeCheckError::InvalidType { span, .. } => &span.path,
        TypeCheckError::IncompatibleType { span, ..} => &span.path,
        TypeCheckError::CantInferType { span } => &span.path,
        TypeCheckError::InvalidDot { span } => &span.path,
    };

    std::fs::read_to_string(path)
      .expect("couldn't open file that was already opened")
  }
}

pub type TypeCheckResult<T> = Result<T, TypeCheckError>;
