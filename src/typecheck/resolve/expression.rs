/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::HashMap;

use super::*;
use crate::{
  aster::intrinsics,
  typecheck::expect_type_of
};

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

          if binding.ty.is_none() {
            let value = binding.value.as_ref().unwrap();

            let value_ty = expect_type_of(value.as_ref())?;

            binding.ty = Some(TypeAST {
              span: value.span(),
              e: value_ty
            });
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

  fn resolve_atom(&mut self, atom: &mut AtomExpressionAST, coerce_to: Option<&Type>) -> TypeCheckResult<Type> {
    match &mut atom.a {
      AtomExpression::Literal(lit) => {
        match &lit.l {
          Literal::UnicodeString(unicode) => {
            let span = lit.span();

            let len = LiteralAST {
              span, l: Literal::IntLiteral(unicode.len().to_string()),
            };

            let array = Type::ArrayOf(
              Some(len),
              Box::new(TypeAST {
                span, e: Type::Intrinsic(intrinsics::U32)
              })
            );

            let array_reference = Type::ConstReferenceTo(Box::new(TypeAST {
              span,
              e: array,
            }));

            atom.out = array_reference;
          },
          Literal::ByteString(text) => {
            let span = lit.span();

            let len = LiteralAST {
              span, l: Literal::IntLiteral(text.len().to_string())
            };

            let array = Type::ArrayOf(
              Some(len),
              Box::new(TypeAST {
                span, e: Type::Intrinsic(intrinsics::U8)
              })
            );

            let array_reference = Type::ConstReferenceTo(
              Box::new(TypeAST {
                span,
                e: array,
              })
            );

            atom.out = array_reference;
          },
          Literal::CString(text) => {
            let span = lit.span();

            // include extra byte for null-terminator
            let size = text.len() + 1;

            let len = LiteralAST {
              span, l: Literal::IntLiteral(size.to_string())
            };

            let array = Type::ArrayOf(
              Some(len),
              Box::new(TypeAST {
                span, e: Type::Intrinsic(intrinsics::U8)
              })
            );

            let array_reference = Type::ConstReferenceTo(
              Box::new(TypeAST {
                span,
                e: array,
              })
            );

            atom.out = array_reference;
          },
          Literal::Char(_) => todo!("resolve char"),
          Literal::ByteChar(_) => todo!("resolve bytechar"),
          Literal::FloatLiteral(_) => todo!("resolve float literal"),
          Literal::IntLiteral(_) => {
            let Some(coerce_to) = coerce_to else {
              todo!("error: int literal has no type coercion");
            };

            if extends(coerce_to, &Type::Intrinsic(intrinsics::U8)) {
              atom.out = coerce_to.clone();
            } else {
              return IncompatibleTypeSnafu {
                span: atom.span(),
                what: "Integer literal",
                with: coerce_to.to_string()
              }.fail();
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

    Ok(atom.out.clone())
  }

  fn resolve_control_flow(&mut self, flow: &mut ControlFlowAST, _coerce_to: Option<&Type>) -> TypeCheckResult<Type> {
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
  }

  fn resolve_binary_operator(&mut self, binary: &mut BinaryOperatorExpressionAST, _coerce_to: Option<&Type>) -> TypeCheckResult<Type> {
    todo!("reso binop")
  }

  fn resolve_unary_operator(&mut self, unary: &mut UnaryOperatorExpressionAST, _coerce_to: Option<&Type>) -> TypeCheckResult<Type> {
    let expr = &mut unary.expr;

    self.stack.push(ScopePointer::Expression(expr.as_mut()));
    self.resolve_expression(expr, None)?;
    self.stack.pop();

    match &mut unary.op {
      UnaryOperator::UnaryPfx(_) => todo!("unarypfxop reso type"),
      UnaryOperator::UnarySfx(UnarySfxOperator::Call { args }) => {
        match expr.type_of() {
          Some(Type::External(external)) => {
            let external = unsafe { &*external };

            let external_len = external.args.len();
            let call_len = args.len();

            if external_len == call_len {
              let mut external_args = external.args.values().collect::<Vec<_>>();
              external_args.sort_by_key(|ty| ty.span().start);

              for (arg, ty) in args.iter_mut().zip(external_args.iter()) {
                let ty = Type::Defined(*ty);
                let coerce_to = Some(&ty);

                self.resolve_expression(arg, coerce_to)?;
              };

              unary.out = Type::Defined(&external.ret);

              Ok(unary.out.clone())
            } else if external_len < call_len && external.varargs {
              todo!("varargs resolve");
            } else {
              let or_more = if external.varargs {
                " or more"
              } else {
                " "
              };

              IncompatibleTypeSnafu {
                span: expr.span(),
                what: "Function signature",
                with: format!("the provided arguments (expected {}{}, got {})", external_len, or_more, call_len),
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
      UnaryOperator::UnarySfx(_) => todo!("unarysfxop reso type"),
    }
  }

  fn resolve_expression(&mut self, expr: &mut Expression, coerce_to: Option<&Type>) -> TypeCheckResult<Type> {
    match expr {
      Expression::Atom(atom) => self.resolve_atom(atom, coerce_to),
      Expression::Block(_) => todo!("resolve block"),
      Expression::SubExpression(_) => todo!("resolve subexpression"),
      Expression::ControlFlow(flow) => self.resolve_control_flow(flow, coerce_to),
      Expression::BinaryOperator(binary) => self.resolve_binary_operator(binary, coerce_to),
      Expression::UnaryOperator(unary) => self.resolve_unary_operator(unary, coerce_to),
    }
  }
}
