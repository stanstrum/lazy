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
  TypeOf,
  extends
};

use crate::aster::{
  ast::*,
  intrinsics
};

impl Checker {
  fn resolve_dot_member(&mut self, ty: &Type, expr: &mut Expression) -> TypeCheckResult<Type> {
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

        let (fqual, members) = match ty {
          Type::Struct(fqual, members) => {
            (fqual, members)
          },
          Type::Defined(ast) => {
            let ast = unsafe { &**ast };

            return self.resolve_dot_member(&ast.e, expr);
          },
          _ => todo!("err for bad type {ty:?}")
        };

        let (memb_ty, idx) = Self::get_struct_member_idx(members, ident)?;

        atom.a = AtomExpression::ValueVariable(
          qual.clone(),
          VariableReference::ResolvedMemberOf(fqual.to_owned(), members.to_owned(), idx)
        );

        atom.out = memb_ty;

        Ok(expr.type_of_expect(expr.span())?)
      },
      Expression::SubExpression(subexpr) => {
        self.resolve_dot_member(ty, &mut subexpr.e)
      },
      _ => InvalidDotSnafu {
        span: expr.span(),
      }.fail()
    }
  }

  pub fn resolve_binary_operator(&mut self, binary: &mut BinaryOperatorExpressionAST, coerce_to: Option<&Type>) -> TypeCheckResult<Type> {
    match binary.op {
      BinaryOperator::Assign => {
        let a = self.resolve_dest_expression(&mut binary.a)?;
        let b = self.resolve_expression(&mut binary.b, Some(&a))?;

        if !extends(&a, &b) {
          return IncompatibleTypeSnafu {
            span: binary.span(),
            what: "Assignment value",
            with: "variable type",
          }.fail();
        };

        binary.out = Type::Intrinsic(intrinsics::VOID);
      },
      BinaryOperator::Add => {
        let out = {
          if let Ok(ty) = self.resolve_expression(&mut binary.a, coerce_to) {
            self.resolve_expression(&mut binary.b, Some(&ty))?
          } else if let Ok(ty) = self.resolve_expression(&mut binary.b, coerce_to) {
            self.resolve_expression(&mut binary.a, Some(&ty))?
          } else {
            return CantInferTypeSnafu {
              span: binary.span(),
            }.fail();
          }
        };

        // todo: search std lib traits & impls...
        binary.out = out;
      },
      BinaryOperator::Dot => {
        let ty = self.resolve_dest_expression(&mut binary.a)?;
        binary.out = self.resolve_dot_member(&ty, &mut binary.b)?;
      },
      _ => todo!("resolve_binary_operator {:?}", binary.op)
    };

    Ok(binary.out.clone())
  }
}
