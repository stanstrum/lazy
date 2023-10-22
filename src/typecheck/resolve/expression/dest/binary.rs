/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::typecheck::{
  Checker,
  TypeCheckResult,
  errors::*,
  TypeOf
};

use crate::aster::ast::*;

impl Checker {
  fn resolve_dest_dot_member(&mut self, ty: &Type, expr: &mut Expression) -> TypeCheckResult<Type> {
    match expr {
      Expression::Atom(atom)
        if matches!(&atom.a, AtomExpression::UnresolvedVariable(_))
      => {
        let AtomExpression::UnresolvedVariable(qual) = &atom.a else {
          unreachable!();
        };

        if qual.parts.len() != 1 {
          return InvalidDotSnafu {
            span: expr.span()
          }.fail();
        };

        let ident = unsafe { qual.parts.first().unwrap_unchecked() };

        let r#struct = match ty {
          Type::Struct(r#struct) => {
            unsafe { &**r#struct }
          },
          Type::Defined(ast) => {
            let ast = unsafe { &**ast };

            return self.resolve_dest_dot_member(&ast.e, expr);
          },
          _ => todo!("err for bad type {ty:?}")
        };

        let (memb_ty, idx) = Self::get_struct_member_idx(r#struct, ident)?;

        atom.a = AtomExpression::DestinationVariable(
          qual.clone(),
          VariableReference::ResolvedMemberOf(r#struct, idx)
        );

        atom.out = memb_ty;

        Ok(expr.type_of_expect(expr.span())?)
      },
      Expression::SubExpression(subexpr) => {
        self.resolve_dest_dot_member(ty, &mut subexpr.e)
      },
      _ => InvalidDotSnafu {
        span: expr.span(),
      }.fail()
    }
  }

  pub fn resolve_dest_binary_operator(&mut self, binary: &mut BinaryOperatorExpressionAST) -> TypeCheckResult<Type> {
    match &binary.op {
      BinaryOperator::Dot => {
        let ty = self.resolve_dest_expression(&mut binary.a)?;
        binary.out = self.resolve_dest_dot_member(&ty, &mut binary.b)?;

        Ok(binary.out.clone())
      },
      other => todo!("resolve_dest_expression binaryoperator {other:?}")
    }
  }
}
