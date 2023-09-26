/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

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

  fn resolve_expression(&mut self, expr: &mut Expression) -> TypeCheckResult<()> {
    match expr {
      Expression::Atom(atom) => {
        match &mut atom.a {
          AtomExpression::Literal(lit) => {
            match &lit.l {
              Literal::UnicodeString(unicode) => {
                let span = lit.span();

                let len = LiteralAST {
                  span, l: Literal::NumericLiteral(unicode.len().to_string()),
                };

                atom.out = Type::ArrayOf(Some(len), Box::new(TypeAST {
                  span, e: Type::Intrinsic(&intrinsics::U32)
                }));
              },
              Literal::ByteString(_) => todo!("resolve bytestr"),
              Literal::CString(_) => todo!("resolve cstr"),
              Literal::Char(_) => todo!("resolve char"),
              Literal::ByteChar(_) => todo!("resolve bytechar"),
              Literal::NumericLiteral(_) => todo!("resolve numeric literal"),
            };
          },
          AtomExpression::Variable(qual, resolved) => {
            *resolved = self.resolve_variable(qual)?;

            atom.out = resolved.r#typeof().expect("couldn't resolve out type for atom");
          },
          AtomExpression::Return(_) => todo!("atom return"),
          AtomExpression::Break(_) => todo!("atom break"),
        };
      },
      Expression::Block(_) => todo!("resolve block"),
      Expression::SubExpression(_) => todo!("resolve subexpression"),
      Expression::ControlFlow(flow) => {
        match &mut flow.e {
          ControlFlow::If(_, _) => todo!("if"),
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
                    qual, _
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
                  };
                };

                todo!("resolve out type of atom");
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
          }
        };
      },
      Expression::UnaryOperator(unary) => {
        let expr = &mut *unary.expr;

        self.stack.push(ScopePointer::Expression(expr));
        self.resolve_expression(expr)?;
        self.stack.pop();
      },
    };

    Ok(())
  }
}
