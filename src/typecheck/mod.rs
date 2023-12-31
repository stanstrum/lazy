/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

pub mod errors;
mod resolve;
mod type_of;

pub use type_of::TypeOf;

use errors::*;

use crate::aster::ast::*;

mod extends;
pub use extends::*;

#[allow(unused)]
#[derive(Clone, Copy)]
enum ScopePointer {
  Namespace(*mut NamespaceAST),
  Function(*mut FunctionAST),
  Block(*mut BlockExpressionAST),
  Impl(*mut Impl),
  MemberFunction(*mut MemberFunctionAST),
  Template(*mut TemplateAST),
}

pub struct Checker {
  stack: Vec<ScopePointer>,
  // impls: Vec<(Type, *const Impl)>,
}

pub fn check(mut global: NamespaceAST) -> TypeCheckResult<NamespaceAST> {
  let mut checker = Checker::new(&mut global);
  checker.resolve_ns(&mut global)?;

  Ok(global)
}
