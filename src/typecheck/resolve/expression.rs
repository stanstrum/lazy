/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::HashMap;

use super::*;
use crate::aster::intrinsics;

impl Checker {
  pub fn resolve_block_expression(&mut self, block: &mut BlockExpressionAST) -> TypeCheckResult<()> {
    for expr in block.children.iter_mut() {
      match expr {
        BlockExpressionChild::Binding(binding) => {
          if binding.ty.is_some() {
            self.resolve_type(binding.ty.as_mut().unwrap())?;
          };

          block.vars.insert(binding.ident.clone(), binding);
        },
        BlockExpressionChild::Expression(expr) => {
          self.stack.push(ScopePointer::Expression(expr));
          self.resolve_expression(expr)?;
          self.stack.pop();
        },
      };
    };

    Ok(())
  }

  fn get_impls_for(&self, ty: Type) -> TypeCheckResult<HashMap<IdentAST, VariableReference>> {
    let mut map = HashMap::<IdentAST, *const MemberFunctionAST>::new();

    for (ty, r#impl) in self.impls.iter() {
      let r#impl = unsafe { &**r#impl };

      let (implemented_ty, methods) = match r#impl {
        Impl::Impl(ImplAST { ty, methods, .. }) => {
          (&ty.e, methods)
        },
        Impl::ImplFor(ImplForAST { ty, methods, .. }) => {
          (&ty.e, methods)
        }
      };

      if extends(ty, implemented_ty) {
        for method in methods.iter() {
          let ident = &method.decl.decl.ident;

          if map.contains_key(ident) {
            let original = unsafe { &**map.get(ident).unwrap() };

            let span_a = original.span();
            let span_b = method.span();

            return DuplicateIdentSnafu {
              text: ident.text.to_owned(),
              a: span_a,
              b: span_b,
            }.fail()
          };

          map.insert(ident.clone(), method);
        };
      };
    };

    let map = map.iter().map(
      |(k, v)|
        (k.to_owned(), VariableReference::ResolvedMemberFunction(unsafe { &**v }))
      ).collect::<HashMap<_, _>>();

    Ok(map)
  }

  fn resolve_expression(&mut self, expr: &mut Expression) -> TypeCheckResult<()> {
    match expr {
      Expression::Atom(atom) => {
        match &mut atom.a {
          AtomExpression::Literal(lit) => {
            match &lit.l {
              Literal::UnicodeString(unicode) => {
                let span = lit.span();

                let len = LiteralAST {
                  span, l: Literal::IntLiteral(unicode.len().to_string()),
                };

                atom.out = Type::ArrayOf(Some(len), Box::new(TypeAST {
                  span, e: Type::Intrinsic(intrinsics::U32)
                }));
              },
              Literal::ByteString(_) => todo!("resolve bytestr"),
              Literal::CString(_) => todo!("resolve cstr"),
              Literal::Char(_) => todo!("resolve char"),
              Literal::ByteChar(_) => todo!("resolve bytechar"),
              Literal::FloatLiteral(_) => todo!("resolve numeric literal"),
              Literal::IntLiteral(_) => todo!("resolve numeric literal"),
            };
          },
          AtomExpression::Variable(qual, resolved) => {
            *resolved = self.resolve_variable(qual)?;

            let out = resolved.type_of();

            if let Some(out) = out {
              atom.out = out;
            } else {
              panic!("failed to resolve atom type `{}`", atom.to_string());
            };
          },
          AtomExpression::Return(_) => todo!("atom return"),
          AtomExpression::Break(_) => todo!("atom break"),
        };
      },
      Expression::Block(_) => todo!("resolve block"),
      Expression::SubExpression(_) => todo!("resolve subexpression"),
      Expression::ControlFlow(flow) => {
        match &mut flow.e {
          ControlFlow::If(cond_body, r#else) => {
            let mut out_ty = None;

            for (cond, body) in cond_body.iter_mut() {
              self.resolve_expression(cond)?;

              self.stack.push(ScopePointer::Block(body));
              self.resolve_block_expression(body)?;
              self.stack.pop();

              if out_ty.is_none() {
                out_ty = body.type_of();
              } else if !extends(&body.type_of().unwrap(), out_ty.as_ref().unwrap()) {
                panic!("doesn't match types in if block")
              };
            };

            if r#else.is_some() {
              let body = r#else.as_mut().unwrap();

              self.stack.push(ScopePointer::Block(body));
              self.resolve_block_expression(body)?;
              self.stack.pop();
            };

            todo!("if")
          },
          ControlFlow::While(cond, body) => {
            self.stack.push(ScopePointer::Expression(&mut **cond));
            self.resolve_expression(cond)?;
            self.stack.pop();

            self.stack.push(ScopePointer::Block(&mut **body));
            self.resolve_block_expression(body)?;
            self.stack.pop();
          },
          ControlFlow::DoWhile(_, _) => todo!("dowhile"),
          ControlFlow::Loop(block) => {
            let block = &mut **block;

            self.stack.push(ScopePointer::Block(block));
            self.resolve_block_expression(block)?;
            self.stack.pop();
          },
        };

        todo!("resolve controlflow");
      },
      Expression::BinaryOperator(binary) => {
        match binary.op {
          BinaryOperator::Dot => {
            let (a, b) = (&mut *binary.a, &mut *binary.b);

            self.stack.push(ScopePointer::Expression(a));
            self.resolve_expression(a)?;
            self.stack.pop();

            match b {
              Expression::Atom(
                AtomExpressionAST {
                  a: AtomExpression::Variable(
                    qual, var_ref
                  ), ..
                }
              ) => {
                let ident = {
                  if qual.parts.len() == 1 {
                    qual.parts.first().unwrap()
                  } else {
                    return InvalidDotSnafu {
                      span: b.span()
                    }.fail();
                  }
                };

                let parent_ty = a.type_of().expect("need to know parent ty");

                let impls = self.get_impls_for(parent_ty)?;

                dbg!(impls.keys());

                if !impls.contains_key(ident) {
                  return UnknownIdentSnafu {
                    text: ident.text.to_owned(),
                    span: qual.span()
                  }.fail();
                };

                *var_ref = impls.get(ident).unwrap().to_owned();
              },
              _ => {
                return InvalidDotSnafu {
                  span: b.span()
                }.fail();
              }
            }
          },
          _ => {
            let (a, b) = (&mut *binary.a, &mut *binary.b);

            self.stack.push(ScopePointer::Expression(a));
            self.resolve_expression(a)?;
            self.stack.pop();

            self.stack.push(ScopePointer::Expression(b));
            self.resolve_expression(b)?;
            self.stack.pop();

            todo!("set out for binop");
          }
        };
      },
      Expression::UnaryOperator(UnaryOperatorExpressionAST { out, expr, op, .. }) => {
        self.stack.push(ScopePointer::Expression(&mut **expr));
        self.resolve_expression(expr)?;
        self.stack.pop();

        let expr_ty = expr.type_of();

        match op {
          UnaryOperator::UnaryPfx(_) => todo!("unarypfxop reso type"),
          UnaryOperator::UnarySfx(_) => todo!("unarysfxop reso type"),
        };

        *out = todo!("set out for unaryop");
      },
    };

    Ok(())
  }
}
