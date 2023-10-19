/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

// use std::collections::HashMap;

use std::collections::HashMap;

use super::*;
use crate::{
  aster::intrinsics,
  typecheck::{
    expect_type_of,
    type_of::*,
    extends::assignable
  }
};

const BOOL_COERCION: Option<&Type> = Some(&Type::Intrinsic(intrinsics::BOOL));

fn get_struct_member_idx(r#struct: &StructAST, ident: &IdentAST) -> TypeCheckResult<(Type, usize)> {
  for (i, (memb_ty, member_ident)) in r#struct.members.iter().enumerate() {
    if ident == member_ident {
      return Ok((Type::Defined(memb_ty), i));
    };
  };

  todo!("error for ident not found");
}

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

  // fn get_impls_for(&self, ty: &Type) -> TypeCheckResult<HashMap<IdentAST, VariableReference>> {
  //   let mut map = HashMap::<IdentAST, *const MemberFunctionAST>::new();

  //   for (implemented_ty, r#impl) in self.impls.iter() {
  //     let r#impl = unsafe { &**r#impl };

  //     let methods = match r#impl {
  //       Impl::Impl(r#impl) => {
  //         &r#impl.methods
  //       },
  //       Impl::ImplFor(impl_for) => {
  //         &impl_for.methods
  //       }
  //     };

  //     if extends(ty, implemented_ty) {
  //       for method in methods.iter() {
  //         let ident = &method.decl.decl.ident;

  //         if map.contains_key(ident) {
  //           let original = unsafe { &**map.get(ident).unwrap() };

  //           let span_a = original.span();
  //           let span_b = method.span();

  //           return DuplicateIdentSnafu {
  //             text: ident.text.to_owned(),
  //             a: span_a,
  //             b: span_b,
  //           }.fail()
  //         };

  //         map.insert(ident.clone(), method);
  //       };
  //     };
  //   };

  //   let map = map.iter().map(
  //     |(k, v)|
  //       (k.to_owned(), VariableReference::ResolvedMemberFunction(unsafe { &**v }))
  //     ).collect::<HashMap<_, _>>();

  //   Ok(map)
  // }

  fn get_qualified_structure<'a>(&'a self, qual: &QualifiedAST) -> TypeCheckResult<&'a Structure> {
    let mut stack = self.stack.iter().filter_map(|ptr| match ptr {
      ScopePointer::Namespace(ns) => {
        Some(unsafe { &**ns })
      },
      _ => None
    }).collect::<Vec<_>>();

    let (last, scopes) = qual.parts.split_last().unwrap();

    for part in scopes {
      let map = &stack.last().unwrap().map;

      match map.get(&part.to_hashable()) {
        Some(Structure::Namespace(ns)) => {
          stack.push(ns);
        },
        Some(Structure::ImportedNamespace { ns, .. }) => {
          let ns = unsafe { &**ns };

          stack.push(ns);
        },
        _ if part.text == "super" => {
          stack.pop();
        },
        Some(_) => {
          return InvalidTypeSnafu {
            text: format!("{} is not a namespace", &part.text),
            span: part.span(),
          }.fail();
        },
        None => {
          return UnknownIdentSnafu {
            text: &part.text,
            span: part.span(),
          }.fail();
        }
      };
    };

    let last_ns = stack.last().unwrap();

    match last_ns.map.get(&last.to_hashable()) {
      Some(structure) => Ok(Self::follow_structure(structure)),
      None => UnknownIdentSnafu {
        text: &last.text,
        span: last.span(),
      }.fail()
    }
  }

  fn resolve_atom(&mut self, atom: &mut AtomExpressionAST, coerce_to: Option<&Type>) -> TypeCheckResult<Type> {
    match &mut atom.a {
      AtomExpression::Literal(lit) => {
        let span = &lit.span;

        match &lit.l {
          Literal::UnicodeString(unicode) => {
            let len = LiteralAST {
              span: span.clone(),
              l: Literal::IntLiteral(unicode.len().to_string()),
            };

            let array = Type::ArrayOf(
              Some(len),
              Box::new(TypeAST {
                span: span.clone(),
                e: Type::Intrinsic(intrinsics::U32)
              })
            );

            let array_reference = Type::ConstReferenceTo(Box::new(TypeAST {
              span: span.clone(),
              e: array,
            }));

            atom.out = array_reference;
          },
          Literal::ByteString(text) => {
            let len = LiteralAST {
              span: span.clone(),
              l: Literal::IntLiteral(text.len().to_string())
            };

            let array = Type::ArrayOf(
              Some(len),
              Box::new(TypeAST {
                span: span.clone(),
                e: Type::Intrinsic(intrinsics::U8)
              })
            );

            let array_reference = Type::ConstReferenceTo(
              Box::new(TypeAST {
                span: span.clone(),
                e: array,
              })
            );

            atom.out = array_reference;
          },
          Literal::CString(text) => {
            // include extra byte for null-terminator
            let size = text.len() + 1;

            let len = LiteralAST {
              span: span.clone(),
              l: Literal::IntLiteral(size.to_string())
            };

            let array = Type::ArrayOf(
              Some(len),
              Box::new(TypeAST {
                span: span.clone(),
                e: Type::Intrinsic(intrinsics::U8)
              })
            );

            let array_reference = Type::ConstReferenceTo(
              Box::new(TypeAST {
                span: span.clone(),
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
      AtomExpression::UnresolvedVariable(qual) => {
        let resolved = self.resolve_variable(qual)?;
        let out = resolved.type_of();

        atom.a = AtomExpression::ValueVariable(qual.to_owned(), resolved);

        if let Some(out) = out {
          atom.out = out;
        } else {
          panic!("failed to resolve atom type `{}`", atom.to_string());
        };
      },
      AtomExpression::StructInitializer(initializer) => {
        let Structure::Struct(r#struct) = self.get_qualified_structure(&initializer.qual)? else {
          return InvalidTypeSnafu {
            text: "Initializer is not a struct",
            span: initializer.qual.span(),
          }.fail();
        };

        atom.out = Type::Struct(r#struct as *const _);

        let init_len = initializer.members.len();
        let struct_len = r#struct.members.len();
        if init_len != struct_len {
          return IncompatibleTypeSnafu {
            span: initializer.span(),
            what: "Struct initializer fields",
            with: format!("struct definition (expected {struct_len}, got {init_len})"),
          }.fail();
        };

        let mut item_map = HashMap::<IdentAST, Type>::new();

        for (ty, ident) in r#struct.members.iter() {
          item_map.insert(ident.to_owned(), Type::Defined(ty));
        };

        for (ident, expr) in initializer.members.iter_mut() {
          let Some(field_ty) = item_map.remove(ident) else {
            todo!("bad field");
          };

          let expr_ty = self.resolve_expression(expr, Some(&field_ty))?;

          if !assignable(&expr_ty, &field_ty) {
            todo!("types dont match")
          };
        };
      },
      _ => todo!("{:#?}", &atom.a),
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
          } else if !assignable(&body.type_of().unwrap(), out_ty.as_ref().unwrap()) {
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

        let r#struct = match ty {
          Type::Struct(r#struct) => {
            unsafe { &**r#struct }
          },
          Type::Defined(ast) => {
            let ast = unsafe { &**ast };

            return self.resolve_dot_member(&ast.e, expr);
          },
          _ => todo!("err for bad type {ty:?}")
        };

        let (memb_ty, idx) = get_struct_member_idx(r#struct, ident)?;

        atom.a = AtomExpression::ValueVariable(
          qual.clone(),
          VariableReference::ResolvedMemberOf(r#struct, idx)
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

  fn resolve_binary_operator(&mut self, binary: &mut BinaryOperatorExpressionAST, coerce_to: Option<&Type>) -> TypeCheckResult<Type> {
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

  fn resolve_unary_operator(&mut self, unary: &mut UnaryOperatorExpressionAST, coerce_to: Option<&Type>) -> TypeCheckResult<Type> {
    let span = unary.span();
    let expr = &mut unary.expr;

    match &mut unary.op {
      UnaryOperator::UnarySfx(UnarySfxOperator::Call { args }) => {
        self.stack.push(ScopePointer::Expression(expr.as_mut()));
        self.resolve_dest_expression(expr)?;
        self.stack.pop();

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

        self.stack.push(ScopePointer::Expression(expr.as_mut()));
        self.resolve_expression(expr, coerce_to.as_ref())?;
        self.stack.pop();

        unary.out = expr.type_of_expect(expr.span())?;

        Ok(unary.out.clone())
      },
      UnaryOperator::UnarySfx(UnarySfxOperator::Cast { to, method }) => {
        self.stack.push(ScopePointer::Expression(expr.as_mut()));
        self.resolve_expression(expr, Some(&Type::Defined(to)))?;
        self.stack.pop();

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

  fn resolve_dest_unary_operator(&mut self, unary: &mut UnaryOperatorExpressionAST) -> TypeCheckResult<Type> {
    match &mut unary.op {
      UnaryOperator::UnarySfx(UnarySfxOperator::Subscript { arg, dest }) => {
        let span = unary.expr.span();

        let ptr_arr_ty = self.resolve_expression(&mut unary.expr, None)?;
        self.resolve_expression(arg, Some(&Type::Intrinsic(intrinsics::USIZE)))?;

        let arr_ty = dereference_type(&ptr_arr_ty, span.clone())?;
        if !is_array(&arr_ty) {
          return IncompatibleTypeSnafu {
            span,
            what: "Expression",
            with: "array index",
          }.fail();
        };

        *dest = true;

        let out = get_element_of(&arr_ty, span)?;
        unary.out = out;

        Ok(unary.out.clone())
      },
      _ => todo!("resolve_dest_unary_operator {:#?}", &unary.op)
    }
  }

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

        let (memb_ty, idx) = get_struct_member_idx(r#struct, ident)?;

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

  fn resolve_dest_binary_operator(&mut self, binary: &mut BinaryOperatorExpressionAST) -> TypeCheckResult<Type> {
    match &binary.op {
      BinaryOperator::Dot => {
        let ty = self.resolve_dest_expression(&mut binary.a)?;
        binary.out = self.resolve_dest_dot_member(&ty, &mut binary.b)?;

        Ok(binary.out.clone())
      },
      other => todo!("resolve_dest_expression binaryoperator {other:?}")
    }
  }

  fn resolve_dest_expression(&mut self, expr: &mut Expression) -> TypeCheckResult<Type> {
    match expr {
      Expression::Atom(atom) => self.resolve_dest_atom(atom),
      Expression::Block(_) => todo!("resolve_dest_expression block"),
      Expression::SubExpression(_) => todo!("resolve_dest_expression subexpression"),
      Expression::ControlFlow(_) => todo!("resolve_dest_expression controlflow"),
      Expression::UnaryOperator(unary) => self.resolve_dest_unary_operator(unary),
      Expression::BinaryOperator(binary) => self.resolve_dest_binary_operator(binary)
    }
  }

  fn resolve_dest_atom(&mut self, atom: &mut AtomExpressionAST) -> TypeCheckResult<Type> {
    let span = atom.span();

    match &mut atom.a {
      AtomExpression::Literal(_) => todo!("resolve_dest_atom literal"),
      AtomExpression::UnresolvedVariable(qual) => {
        let var_ref = self.resolve_variable(qual)?;
        let ty = var_ref.type_of_expect(span)?;

        atom.a = AtomExpression::DestinationVariable(qual.to_owned(), var_ref);
        atom.out = ty.clone();

        Ok(ty)
      },
      AtomExpression::ValueVariable(..) => todo!("resolve_dest_atom valuevariable"),
      AtomExpression::DestinationVariable(..) => todo!("resolve_dest_atom destinationvariable"),
      AtomExpression::Return(_) => todo!("resolve_dest_atom return"),
      AtomExpression::Break(_) => todo!("resolve_dest_atom break"),
      other => todo!("resolve_dest_atom {other:?}")
    }
  }

  fn resolve_expression(&mut self, expr: &mut Expression, coerce_to: Option<&Type>) -> TypeCheckResult<Type> {
    match expr {
      Expression::Atom(atom) => self.resolve_atom(atom, coerce_to),
      Expression::Block(_) => todo!("resolve block"),
      Expression::SubExpression(subexpr) => {
        subexpr.out = self.resolve_expression(&mut subexpr.e, coerce_to)?;

        Ok(subexpr.out.clone())
      },
      Expression::ControlFlow(flow) => self.resolve_control_flow(flow, coerce_to),
      Expression::BinaryOperator(binary) => self.resolve_binary_operator(binary, coerce_to),
      Expression::UnaryOperator(unary) => self.resolve_unary_operator(unary, coerce_to),
    }
  }
}
