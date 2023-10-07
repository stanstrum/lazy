/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::HashMap;

use super::*;
use crate::aster::intrinsics;

const BOOL_COERCION: Option<&Type> = Some(&Type::Intrinsic(intrinsics::BOOL));

impl Checker {
  pub fn resolve_block_expression(&mut self, block: &mut BlockExpressionAST, coerce_to: Option<&Type>) -> TypeCheckResult<()> {
    let len = block.children.len();

    if !block.returns_last {
      block.out = Type::Intrinsic(intrinsics::VOID);
    };

    for (i, expr) in block.children.iter_mut().enumerate() {
      match expr {
        BlockExpressionChild::Binding(binding) => {
          if binding.ty.is_some() {
            self.resolve_type(binding.ty.as_mut().unwrap())?;
          };

          if binding.value.is_some() {
            self.resolve_expression(
              binding.value.as_mut().unwrap(),
              binding.ty.as_ref().map(|ast| &ast.e)
            )?;
          };

          block.vars.insert(binding.ident.clone(), binding);
        },
        BlockExpressionChild::Expression(expr) => {
          self.stack.push(ScopePointer::Expression(expr));

          if i + 1 == len && block.returns_last {
            self.resolve_expression(expr, coerce_to)?;

            block.out = expr.type_of()
              .expect("resolve expression did resolve out type");
          } else {
            self.resolve_expression(expr, None)?;
          };

          self.stack.pop();
        },
      };
    };

    Ok(())
  }

  fn get_impls_for(&self, ty: &Type) -> TypeCheckResult<HashMap<IdentAST, VariableReference>> {
    let mut map = HashMap::<IdentAST, *const MemberFunctionAST>::new();

    for (implemented_ty, r#impl) in self.impls.iter() {
      let r#impl = unsafe { &**r#impl };

      let methods = match r#impl {
        Impl::Impl(r#impl) => {
          &r#impl.methods
        },
        Impl::ImplFor(impl_for) => {
          &impl_for.methods
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

  fn resolve_expression(&mut self, expr: &mut Expression, coerce_to: Option<&Type>) -> TypeCheckResult<()> {
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
              Literal::ByteString(text) => {
                let span = lit.span();

                let len = LiteralAST {
                  span, l: Literal::IntLiteral(text.len().to_string())
                };

                atom.out = Type::ArrayOf(Some(len), Box::new(TypeAST {
                  span, e: Type::Intrinsic(intrinsics::U8)
                }));
              },
              Literal::CString(_) => todo!("resolve cstr"),
              Literal::Char(_) => todo!("resolve char"),
              Literal::ByteChar(_) => todo!("resolve bytechar"),
              Literal::FloatLiteral(_) => todo!("resolve float literal"),
              Literal::IntLiteral(_) => {
                let Some(coerce_to) = coerce_to else {
                  todo!("error: int literal has no type coercion");
                };

                // U8,
                // U16,
                // U32,
                // U64,
                // USIZE,
                // I8,
                // I16,
                // I32,
                // I64,
                // ISIZE,

                if extends(coerce_to, &Type::Intrinsic(intrinsics::I32)) {
                  atom.out = coerce_to.clone();
                } else {
                  todo!("error: int literal coercion failed");
                };
              },
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
              self.resolve_expression(cond, BOOL_COERCION)?;

              self.stack.push(ScopePointer::Block(body));
              self.resolve_block_expression(body, None)?;
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
              self.resolve_block_expression(body, None)?;
              self.stack.pop();
            };

            todo!("if")
          },
          ControlFlow::While(cond, body) => {
            self.stack.push(ScopePointer::Expression(&mut **cond));
            self.resolve_expression(cond, BOOL_COERCION)?;
            self.stack.pop();

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
      },
      Expression::BinaryOperator(binary) => {
        match binary.op {
          BinaryOperator::Dot => {
            let (a, b) = (&mut *binary.a, &mut *binary.b);

            self.stack.push(ScopePointer::Expression(a));
            self.resolve_expression(a, None)?;
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

                let impls = self.get_impls_for(&parent_ty)?;

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
            self.resolve_expression(a, None)?;
            self.stack.pop();

            self.stack.push(ScopePointer::Expression(b));
            self.resolve_expression(b, None)?;
            self.stack.pop();

            todo!("set out for binop");
          }
        };
      },
      Expression::UnaryOperator(UnaryOperatorExpressionAST { expr, op, .. }) => {
        self.stack.push(ScopePointer::Expression(&mut **expr));
        self.resolve_expression(expr, None)?;
        self.stack.pop();

        match op {
          UnaryOperator::UnaryPfx(_) => todo!("unarypfxop reso type"),
          UnaryOperator::UnarySfx(_) => todo!("unarysfxop reso type"),
        };
      },
    };

    Ok(())
  }
}
