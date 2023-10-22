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
  assignable,
  type_of::dereference_type
};

use crate::aster::{
  ast::*,
  intrinsics
};

impl Checker {
  pub fn resolve_unary_operator(&mut self, unary: &mut UnaryOperatorExpressionAST, coerce_to: Option<&Type>) -> TypeCheckResult<Type> {
    let span = unary.span();
    let expr = &mut unary.expr;

    match &mut unary.op {
      UnaryOperator::UnarySfx(UnarySfxOperator::Call { args }) => {
        self.resolve_dest_expression(expr)?;

        match expr.type_of() {
          Some(Type::External(external)) => {
            let external = unsafe { &*external };

            let external_len = external.args.len();
            let call_len = args.len();

            let mut external_args = external.args.values().collect::<Vec<_>>();
            external_args.sort_by_key(|ty| ty.span().start);

            if external_len == call_len {
              for (arg, ty) in args.iter_mut().zip(external_args.iter()) {
                let ty = &ty.e;
                let coerce_to = Some(ty);

                let arg_ty = self.resolve_expression(arg, coerce_to)?;

                if !assignable(&arg_ty, ty) {
                  return IncompatibleTypeSnafu {
                    span: arg.span(),
                    what: "Argument",
                    with: format!("function signature (expected {}, got {})", ty.to_string(), arg_ty.to_string()),
                  }.fail();
                };
              };

              unary.out = Type::Defined(&external.ret);

              Ok(unary.out.clone())
            } else if external_len < call_len && external.varargs {
              for i in 0..call_len {
                let arg = &mut args[i];
                let coerce_to = external_args.get(i)
                  .map(|ty| &ty.e);

                self.resolve_expression(arg, coerce_to)?;
              };

              unary.out = Type::Defined(&external.ret);

              Ok(unary.out.clone())
            } else {
              let or_more = if external.varargs {
                " or more"
              } else {
                ""
              };

              IncompatibleTypeSnafu {
                span,
                what: "Function signature",
                with: format!("the provided arguments (expected {}{}, got {})", external_len, or_more, call_len),
              }.fail()
            }
          },
          Some(Type::Function(func)) => {
            let func = unsafe { &*func };

            let func_len = func.decl.args.len();
            let call_len = args.len();

            let mut func_args = func.decl.args.values().collect::<Vec<_>>();
            func_args.sort_by_key(|ty| ty.span().start);

            if func_len == call_len {
              for (arg, ty) in args.iter_mut().zip(func_args.iter()) {
                let ty = &ty.e;
                let coerce_to = Some(ty);

                let arg_ty = self.resolve_expression(arg, coerce_to)?;

                if !assignable(&arg_ty, ty) {
                  return IncompatibleTypeSnafu {
                    span: arg.span(),
                    what: "Argument",
                    with: format!("function signature (expected {}, got {})", ty.to_string(), arg_ty.to_string()),
                  }.fail();
                };
              };

              unary.out = Type::Defined(&func.decl.ret);

              Ok(unary.out.clone())
            } else {
              IncompatibleTypeSnafu {
                span,
                what: "Function signature",
                with: format!("the provided arguments (expected {}, got {})", func_len, call_len),
              }.fail()
            }
          },
          Some(_) => {
            IncompatibleTypeSnafu {
              span: expr.span(),
              what: "Expression",
              with: "function call",
            }.fail()
          },
          None => panic!("couldn't resolve expr for sfx operator"),
        }
      },
      UnaryOperator::UnaryPfx(UnaryPfxOperator::Ref) => {
        let coerce_to = if let Some(coerce_to) = coerce_to {
          Some(dereference_type(coerce_to, expr.span())?)
        } else {
          None
        };

        self.resolve_expression(expr, coerce_to.as_ref())?;

        unary.out = expr.type_of_expect(expr.span())?;

        Ok(unary.out.clone())
      },
      UnaryOperator::UnarySfx(UnarySfxOperator::Cast { to, method }) => {
        self.resolve_expression(expr, Some(&Type::Defined(to)))?;

        self.resolve_type(to)?;
        unary.out = to.e.to_owned();

        let expr_ty = expr.type_of_expect(expr.span())?;

        let mut from_ptr = &expr_ty;
        let mut to_ptr = &to.e;

        loop {
          match (from_ptr, to_ptr) {
            (Type::Defined(defined), _) => {
              from_ptr = unsafe { &(**defined).e };
            },
            (_, Type::Defined(defined)) => {
              to_ptr = unsafe { &(**defined).e };
            },
            (Type::Intrinsic(intrinsics::USIZE), Type::Intrinsic(intrinsics::I32)) => {
              *method = Some(CastMethod::Truncate);

              break Ok(Type::Defined(to));
            },
            (Type::Intrinsic(intrinsics::I32), Type::Intrinsic(intrinsics::USIZE)) => {
              *method = Some(CastMethod::ZeroExtend);

              break Ok(Type::Defined(to));
            },
            (a, b) if assignable(a, b) => {
              *method = None;

              break Ok(Type::Defined(to));
            },
            _ => {
              println!("can't find cast method for {from_ptr:?} to {to_ptr:?}");

              break IncompatibleTypeSnafu {
                span,
                what: "Casted value",
                with: to.to_string(),
              }.fail()
            }
          }
        }
      },
      UnaryOperator::UnaryPfx(op) => todo!("unarypfxop reso type {op:#?}"),
      UnaryOperator::UnarySfx(op) => todo!("unarysfxop reso type {op:#?}"),
    }
  }
}
