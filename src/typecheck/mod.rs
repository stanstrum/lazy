/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

pub mod errors;
mod resolve;

use errors::*;

use crate::aster::ast::*;

#[derive(Clone, Copy)]
enum ScopePointer {
  Namespace(*mut NamespaceAST),
  Function(*mut FunctionAST),
  Block(*mut BlockExpressionAST),
  Expression(*mut Expression),
  Impl(*mut Impl),
  MemberFunction(*mut MemberFunctionAST)
}

impl ScopePointer {
  pub fn new_ns(ptr: *mut NamespaceAST) -> Self {
    Self::Namespace(ptr)
  }

  pub fn new_fn(ptr: *mut FunctionAST) -> Self {
    Self::Function(ptr)
  }

  pub fn new_expr(ptr: *mut Expression) -> Self {
    Self::Expression(ptr)
  }

  pub fn new_impl(ptr: *mut Impl) -> Self {
    Self::Impl(ptr)
  }

  pub fn new_member_fn(ptr: *mut MemberFunctionAST) -> Self {
    Self::MemberFunction(ptr)
  }
}

pub struct Checker {
  stack: Vec<ScopePointer>,
  impls: Vec<(Type, *const Impl)>,
}

pub fn check(mut global: NamespaceAST) -> TypeCheckResult<NamespaceAST> {
  let mut checker = Checker::new(&mut global);
  checker.resolve_ns(&mut global)?;

  Ok(global)
}
