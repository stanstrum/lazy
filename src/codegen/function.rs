/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use inkwell::values::{BasicValue, FunctionValue};

use crate::aster::ast::*;
use super::{
  Codegen,
  CodeGenResult
};

impl<'a, 'ctx> Codegen<'a, 'ctx> {
  pub fn declare_function(&mut self, func: &FunctionAST) -> CodeGenResult<FunctionValue<'ctx>> {
    let decl = &func.decl;

    let ret_ty = self.generate_type(&decl.ret.e)?;

    let mut args = decl
      .args
      .values()
      .collect::<Vec<_>>();

    args.sort_by_key(|ty_ast| ty_ast.span().start);

    let args = args.iter().map(
        |ast| self.generate_type(&ast.e)
      ).collect::<Result<Vec<_>, _>>()?;

    let args = args
      .iter()
      .map(|ty| ty.to_basic_metadata())
      .collect::<Vec<_>>();

    let func_ty = ret_ty.fn_type(args.as_slice(), false);

    let name = &func.decl.ident.text;
    Ok(self.module.add_function(name, func_ty, None))
  }

  pub fn generate_function(&mut self, ast: &FunctionAST, value: FunctionValue<'ctx>) -> CodeGenResult<()> {
    let block = self.context.append_basic_block(value, "entry");
    self.builder.position_at_end(block);

    let returned = self.generate_block(&ast.body)?;

    // this is very strange
    let returned = returned
      .as_ref()
      .map(
        |val| val as &dyn BasicValue<'ctx>
      );

    self.builder.build_return(returned);

    Ok(())
  }
}
