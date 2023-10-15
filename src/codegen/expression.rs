/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use inkwell::values::{
  BasicValueEnum,
  AnyValueEnum,
  AnyValue
};
use crate::{
  aster::{
    ast::*,
    intrinsics
  },
  typecheck::{
    extends,
    TypeOf
  }
};

use super::{
  Codegen,
  CodeGenResult
};

impl<'a, 'ctx> Codegen<'a, 'ctx> {
  fn generate_value_variable(&mut self, var_ref: &VariableReference) -> CodeGenResult<AnyValueEnum<'ctx>> {
    let value = self.var_map.get(var_ref)
      .unwrap_or_else(|| panic!("unresolved value var ref {var_ref:#?}"));

    match var_ref {
      VariableReference::ResolvedArgument(_) => {
        Ok(value.as_any_value_enum())
      },
      VariableReference::ResolvedVariable(_) => {
        let load = self.builder.build_load(
          value.into_pointer_value(),
          "load_variable"
        );

        Ok(load.as_any_value_enum())
      },
      _ => todo!("generate_value_variable {var_ref:#?}")
    }
  }

  fn generate_dest_variable(&mut self, var_ref: &VariableReference) -> CodeGenResult<AnyValueEnum<'ctx>> {
    let value = self.var_map.get(var_ref)
      .unwrap_or_else(|| panic!("unresolved dest var ref {var_ref:?}"));

    match var_ref {
      VariableReference::ResolvedExternal(_)
      | VariableReference::ResolvedFunction(_)
      | VariableReference::ResolvedVariable(_) => {
        // just return the pointer for the dest
        Ok(value.as_any_value_enum())
      },
      _ => todo!("generate_dest_variable {var_ref:#?}")
    }
  }

  fn generate_atom(&mut self, ast: &AtomExpressionAST) -> CodeGenResult<Option<AnyValueEnum<'ctx>>> {
    Ok(match &ast.a {
      AtomExpression::Literal(lit) => Some(
        self.generate_literal(lit, &ast.out)?
        .as_any_value_enum()
      ),
      AtomExpression::ValueVariable(var_ref) => Some(
        self.generate_value_variable(var_ref)?
      ),
      AtomExpression::DestinationVariable(var_ref) => Some(
        self.generate_dest_variable(var_ref)?
      ),
      AtomExpression::Return(_) => todo!(),
      AtomExpression::Break(_) => todo!(),
      AtomExpression::UnresolvedVariable(qual) =>
        panic!("unresolved var ref {} ({}:{})",
          qual.to_string(),
          qual.span.start,
          qual.span.end
        )
    })
  }

  pub fn generate_expr(&mut self, ast: &Expression) -> CodeGenResult<Option<AnyValueEnum<'ctx>>> {
    match ast {
      Expression::Atom(ast) => self.generate_atom(ast),
      Expression::Block(_) => todo!("generate_expr block"),
      Expression::SubExpression(_) => todo!("generate_expr subexpression"),
      Expression::ControlFlow(_) => todo!("generate_expr controlflow"),
      Expression::BinaryOperator(binary) => self.generate_binary_operator(binary),
      Expression::UnaryOperator(unary) => {
        self.generate_unary_operator(unary)
      },
    }
  }

  fn generate_block_child(&mut self, ast: &BlockExpressionChild) -> CodeGenResult<Option<BasicValueEnum<'ctx>>> {
    match ast {
      BlockExpressionChild::Binding(binding) => {
        self.generate_binding(binding)?;

        Ok(None)
      },
      BlockExpressionChild::Expression(expr) => {
        let value = self.generate_expr(expr)?;

        if !extends(&expr.type_of().unwrap(), &Type::Intrinsic(intrinsics::VOID)) {
          Ok(
            value.map(|value| BasicValueEnum::try_from(dbg!(value)).unwrap())
          )
        } else {
          Ok(None)
        }
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

    Ok(
      self.generate_expr(last)?
        .map(|value| BasicValueEnum::try_from(value).unwrap())
    )
  }

  pub fn generate_block(&mut self, ast: &BlockExpressionAST) -> CodeGenResult<Option<BasicValueEnum<'ctx>>> {
    if ast.returns_last {
      return self.generate_block_returns_last(ast);
    };

    for child in ast.children.iter() {
      self.generate_block_child(child)?;
    };

    Ok(None)
  }
}
