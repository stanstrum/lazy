/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::typecheck::{
  Checker,
  TypeCheckResult,
  ScopePointer,
  TypeOf,
  assignable
};

use crate::aster::{
  ast::*,
  intrinsics
};

const BOOL_COERCION: Option<&Type> = Some(&Type::Intrinsic(intrinsics::BOOL));

impl Checker {
  pub fn resolve_control_flow(&mut self, flow: &mut ControlFlowAST, _coerce_to: Option<&Type>) -> TypeCheckResult<Type> {
    match &mut flow.e {
      ControlFlow::If(cond_body, r#else) => {
        let mut out_ty = None;

        for (cond, body) in cond_body.iter_mut() {
          self.resolve_expression(cond, BOOL_COERCION)?;

          self.stack.push(ScopePointer::Block(body));
          self.resolve_block_expression(body, None)?;
          self.stack.pop();

          if out_ty.is_none() {
            out_ty = body.type_of();
          } else if !assignable(&body.type_of().unwrap(), out_ty.as_ref().unwrap()) {
            panic!("doesn't match types in if block")
          };
        };

        if r#else.is_some() {
          let body = r#else.as_mut().unwrap();

          self.stack.push(ScopePointer::Block(body));
          self.resolve_block_expression(body, None)?;
          self.stack.pop();
        };

        todo!("if")
      },
      ControlFlow::While(cond, body) => {
        self.resolve_expression(cond, BOOL_COERCION)?;

        self.stack.push(ScopePointer::Block(&mut **body));
        self.resolve_block_expression(body, None)?;
        self.stack.pop();
      },
      ControlFlow::DoWhile(_, _) => todo!("dowhile"),
      ControlFlow::Loop(block) => {
        let block = &mut **block;

        self.stack.push(ScopePointer::Block(block));
        self.resolve_block_expression(block, None)?;
        self.stack.pop();
      },
    };

    todo!("resolve controlflow");
  }
}
