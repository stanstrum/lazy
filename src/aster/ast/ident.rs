/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use super::*;
use crate::make_get_span;

#[derive(Debug, Clone)]
pub struct KeywordAST {
  pub span: Span,
}

#[derive(Debug, Clone)]
pub struct IdentAST {
  pub span: Span,
  pub text: String,
}

#[derive(Debug, Clone)]
pub struct QualifiedAST {
  pub span: Span,
  pub parts: Vec<IdentAST>,
}

#[derive(Debug, Clone)]
pub struct FullyQualifiedIdentAST {
  pub span: Span,
  pub ident: IdentAST,
  pub generics: Option<Vec<TypeAST>>
}

#[derive(Debug, Clone)]
pub struct FullyQualifiedAST {
  pub span: Span,
  pub parts: Vec<FullyQualifiedIdentAST>
}

impl QualifiedAST {
  pub fn to_hashable(&self) -> String {
    self.parts
      .iter()
      .map(|ident| ident.text.to_owned())
      .collect::<Vec<String>>()
      .join("::")
  }
}

impl std::hash::Hash for IdentAST {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.text.hash(state);
  }
}

impl std::cmp::PartialEq for IdentAST {
  fn eq(&self, other: &Self) -> bool {
    self.text == other.text
  }
}

impl std::cmp::Eq for IdentAST {}

make_get_span![
  KeywordAST,
  IdentAST,
  QualifiedAST,
  FullyQualifiedIdentAST,
  FullyQualifiedAST
];
