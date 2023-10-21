/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

mod namespace;
mod function;
mod r#type;
mod expression;
mod variable;
mod r#impl;

pub use super::TypeOf;

use super::{
  Checker,
  ScopePointer,
  errors::*
};

use crate::aster::ast::*;

pub use super::{
  extends,
  assignable
};

trait IsResolved {
  fn is_resolved(&self) -> bool;
}

impl IsResolved for TypeAST {
  fn is_resolved(&self) -> bool {
    self.e.is_resolved()
  }
}

fn is_valid_array_length(lit: &LiteralAST) -> bool {
  matches!(&lit.l, Literal::IntLiteral(_))
}

impl IsResolved for Type {
  fn is_resolved(&self) -> bool {
    match self {
      Type::Intrinsic(_)
      | Type::Struct(_)
      | Type::Function(_)
      | Type::External(_) => true,
      Type::ConstReferenceTo(ast)
      | Type::MutReferenceTo(ast)
      | Type::ConstPtrTo(ast)
      | Type::MutPtrTo(ast) => ast.is_resolved(),
      Type::ArrayOf(lit, ty) =>
        lit.as_ref().is_some_and(is_valid_array_length) && ty.is_resolved(),
      Type::Defined(ast) => unsafe { (**ast).is_resolved() },
      Type::Unknown(_) => false,
      Type::UnresolvedLiteral(_)
      | Type::Unresolved => false,
    }
  }
}

impl Checker {
  pub fn new(global: &mut NamespaceAST) -> Self {
    Self {
      stack: vec![
        ScopePointer::Namespace(global)
      ],
      // impls: vec![]
    }
  }
}
