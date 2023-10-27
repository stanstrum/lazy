/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::HashMap;

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
  fn generate_value_variable(&mut self, var_ref: &VariableReference, wrt: Option<AnyValueEnum<'ctx>>) -> CodeGenResult<AnyValueEnum<'ctx>> {
    match var_ref {
      VariableReference::ResolvedVariable(binding) => {
        let binding = unsafe { &**binding };

        let value = self.generate_dest_variable(var_ref, None)?
          .into_pointer_value();

        let load = self.builder.build_load(
          value,
          format!("load_{}", binding.ident.to_hashable()).as_str()
        );

        Ok(load.as_any_value_enum())
      },
      VariableReference::ResolvedArgument(_) => {
        let value = self.get_var_ref(var_ref)
          .unwrap_or_else(|| panic!("unresolved value var ref {var_ref:#?}"));

        Ok(value.as_any_value_enum())
      },
      VariableReference::ResolvedMemberOf(fqual, members, idx) => {
        let ptr = self.generate_dest_variable(var_ref, wrt)?
          .into_pointer_value();

        // let r#struct_ast = unsafe { &**r#struct };
        // let ident = &r#struct_ast.ident;
        let (_, member_ident) = unsafe { members.get_unchecked(*idx) };

        let load = self.builder.build_load(
          ptr,
          format!("load_{}.{}",
            fqual.to_hashable(),
            member_ident.to_hashable()
          ).as_str()
        );

        Ok(load.as_any_value_enum())
      },
      _ => todo!("generate_value_variable {var_ref:#?}")
    }
  }

  fn generate_dest_variable(&mut self, var_ref: &VariableReference, wrt: Option<AnyValueEnum<'ctx>>) -> CodeGenResult<AnyValueEnum<'ctx>> {
    match var_ref {
      VariableReference::ResolvedExternal(_)
        | VariableReference::ResolvedFunction(_)
        | VariableReference::ResolvedVariable(_)
      => {
        let value = self.get_var_ref(var_ref)
          .unwrap_or_else(|| panic!("unresolved dest var ref {var_ref:#?}"));

        // just return the pointer for the dest
        Ok(value.as_any_value_enum())
      },
      VariableReference::ResolvedMemberOf(fqual, members, idx) => {
        let wrt = wrt
          .expect("MemberOf needs a wrt value")
          .into_pointer_value();

        // let r#struct_ast = unsafe { &**r#struct_ast };
        // let ident = &r#struct_ast.ident;
        let (_, member_ident) = unsafe { members.get_unchecked(*idx) };

        let ptr = self.builder.build_struct_gep(
          wrt,
          *idx as u32,
          format!("{}.{}",
            fqual.to_hashable(),
            member_ident.to_hashable()
          ).as_str(),
        ).expect("GEP out of bounds");

        Ok(ptr.as_any_value_enum())
      },
      _ => todo!("generate_dest_variable {var_ref:#?}")
    }
  }

  fn generate_atom(&mut self, ast: &AtomExpressionAST, wrt: Option<AnyValueEnum<'ctx>>) -> CodeGenResult<Option<AnyValueEnum<'ctx>>> {
    Ok(match &ast.a {
      AtomExpression::Literal(lit) => Some(
        self.generate_literal(lit, &ast.out)?
          .as_any_value_enum()
      ),
      AtomExpression::ValueVariable(_, var_ref) => Some(
        self.generate_value_variable(var_ref, wrt)?
      ),
      AtomExpression::DestinationVariable(_, var_ref) => Some(
        self.generate_dest_variable(var_ref, wrt)?,
      ),
      AtomExpression::Return(_) => todo!(),
      AtomExpression::Break(_) => todo!(),
      AtomExpression::UnresolvedVariable(qual) =>
        panic!("unresolved var ref {} ({}:{})",
          qual.to_string(),
          qual.span.start,
          qual.span.end
        ),
      AtomExpression::StructInitializer(initializer) => {
        let map: HashMap<IdentAST, Expression> = initializer.members.clone().into_iter().collect();

        let Type::Struct(_, members) = &ast.out else { unreachable!() };
        let ty = self.generate_type(&ast.out)?.to_basic_metadata();

        let mut values = Vec::<BasicValueEnum>::new();
        for  (_, member_ident) in members.iter() {
          let curr_ast = map.get(member_ident).unwrap();
          let curr: BasicValueEnum<'ctx> = self.generate_expr(curr_ast, None)?
            .expect("generate_expr didn't return for struct initializer field")
            .try_into().unwrap();

          values.push(curr);
        };

        let struct_value = ty.into_struct_type().const_named_struct(values.as_slice());

        Some(struct_value.as_any_value_enum())
      },
      #[allow(unused)]
      other => todo!("generate_atom {other:?}")
    })
  }

  pub fn generate_expr(&mut self, ast: &Expression, wrt: Option<AnyValueEnum<'ctx>>) -> CodeGenResult<Option<AnyValueEnum<'ctx>>> {
    match ast {
      Expression::Atom(ast) => self.generate_atom(ast, wrt),
      Expression::Block(block) => {
        let result = self.generate_block(block)?;

        Ok(
          result.map(|x| x.as_any_value_enum())
        )
      },
      Expression::SubExpression(subexpr) => self.generate_expr(&subexpr.e, wrt),
      Expression::ControlFlow(_) => todo!("generate_expr controlflow"),
      Expression::BinaryOperator(binary) => self.generate_binary_operator(binary, wrt),
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
        let value = self.generate_expr(expr, None)?;

        if !extends(&expr.type_of().unwrap(), &Type::Intrinsic(intrinsics::VOID)) {
          Ok(
            value.map(|value| BasicValueEnum::try_from(value).unwrap())
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
      self.generate_expr(last, None)?
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
