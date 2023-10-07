/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use inkwell::values::BasicValueEnum;
use crate::aster::ast::{BlockExpressionAST, BlockExpressionChild, AtomExpressionAST, AtomExpression, VariableReference, Expression};

use super::{
  Codegen,
  CodeGenResult
};

impl<'a, 'ctx> Codegen<'a, 'ctx> {
  fn generate_atom(&mut self, ast: &AtomExpressionAST) -> CodeGenResult<Option<BasicValueEnum<'ctx>>> {
    Ok(match &ast.a {
      AtomExpression::Literal(lit) => Some(self.generate_literal(lit, &ast.out)?),
      AtomExpression::Variable(qual, var_ref)
        if matches!(var_ref, VariableReference::ResolvedVariable(_)) =>
      {
        let name = qual.parts.last().unwrap().text.as_str();

        let ptr = self.var_map
          .get(var_ref)
          .expect("we don't have this variablereference")
          .into_pointer_value();

        let value = self.builder.build_load(ptr, name);

        Some(value)
      },
      AtomExpression::Variable(_, var_ref)
        if matches!(var_ref, VariableReference::ResolvedArgument(_)) =>
      {
        let value = self.var_map.get(var_ref)
          .expect("we don't have this variablereference")
          .to_owned();

        Some(value)
      },
      AtomExpression::Variable(_, _) => todo!(),
      AtomExpression::Return(_) => todo!(),
      AtomExpression::Break(_) => todo!(),
    })
  }

  pub fn generate_expr(&mut self, ast: &Expression) -> CodeGenResult<Option<BasicValueEnum<'ctx>>> {
    match ast {
      Expression::Atom(ast) => self.generate_atom(ast),
      Expression::Block(_) => todo!("generate_expr block"),
      Expression::SubExpression(_) => todo!("generate_expr subexpression"),
      Expression::ControlFlow(_) => todo!("generate_expr controlflow"),
      Expression::BinaryOperator(_) => todo!("generate_expr binaryoperator"),
      Expression::UnaryOperator(_) => todo!("generate_expr unaryoperator"),
    }
  }

  fn generate_block_child(&mut self, ast: &BlockExpressionChild) -> CodeGenResult<Option<BasicValueEnum<'ctx>>> {
    match ast {
      BlockExpressionChild::Binding(binding) => {
        self.generate_binding(binding)?;

        Ok(None)
      },
      BlockExpressionChild::Expression(expr) => {
        self.generate_expr(expr)
      },
    }
  }

  fn generate_block_returns_last(&mut self, ast: &BlockExpressionAST) -> CodeGenResult<Option<BasicValueEnum<'ctx>>> {
    let Some((last, all_but_last)) = ast.children.split_last() else {
      return Ok(None);
    };

    let BlockExpressionChild::Expression(last) = last else {
      unreachable!("last child of returns_last block expression was a binding");
    };

    for child in all_but_last.iter() {
      self.generate_block_child(child)?;
    };

    self.generate_expr(last)
  }

  pub fn generate_block(&mut self, ast: &BlockExpressionAST) -> CodeGenResult<Option<BasicValueEnum<'ctx>>> {
    if ast.returns_last {
      return self.generate_block_returns_last(ast);
    };

    for child in ast.children.iter() {
      self.generate_block_child(&child)?;
    };

    Ok(None)
  }
}
