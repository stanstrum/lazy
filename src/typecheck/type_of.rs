/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use super::*;
use crate::aster::intrinsics;

pub trait TypeOf {
  fn type_of(&self) -> Option<Type>;

  fn type_of_expect(&self, span: Span) -> TypeCheckResult<Type>
  where Self: std::string::ToString {
    if let Some(ty) = self.type_of() {
      Ok(ty)
    } else {
      InvalidTypeSnafu {
        text: format!("Unable to resolve type of: {}", self.to_string()),
        span
      }.fail()
    }
  }
}

pub fn dereference_type(ty: &Type, span: Span) -> TypeCheckResult<Type> {
  match ty {
    Type::Defined(ast) => {
      let ast = unsafe { &**ast };

      dereference_type(&ast.e, span)
    },
    Type::ConstReferenceTo(ast) => {
      Ok(ast.e.to_owned())
    },
    _ => InvalidTypeSnafu {
      text: format!("Unable to dereference type: {}", ty.to_string()),
      span
    }.fail()
  }
}

pub fn is_array(ty: &Type) -> bool {
  match ty {
    Type::Defined(ast) => {
      let ast = unsafe { &**ast };

      is_array(&ast.e)
    },
    Type::ArrayOf(_, _) => true,
    _ => false
  }
}

pub fn get_element_of(ty: &Type, span: Span) -> TypeCheckResult<Type> {
  match ty {
    Type::Defined(ast) => {
      let ast = unsafe { &**ast };

      get_element_of(&ast.e, span)
    },
    Type::ArrayOf(_, ast) => Ok(ast.e.clone()),
    _ => InvalidTypeSnafu {
      text: "Non-array type passed to get_element_of",
      span,
    }.fail()
  }
}

impl TypeOf for BlockExpressionAST {
  fn type_of(&self) -> Option<Type> {
    if self.returns_last {
      match self.children.last().unwrap() {
        BlockExpressionChild::Binding(_) => panic!("returns last but last is a binding..."),
        BlockExpressionChild::Expression(expr) => expr.type_of(),
      }
    } else {
      Some(Type::Intrinsic(intrinsics::VOID))
    }
  }
}

impl TypeOf for BinaryOperatorExpressionAST {
  fn type_of(&self) -> Option<Type> {
    match &self.op {
      BinaryOperator::Add => {
        let a = self.a.type_of();
        let b = self.b.type_of();

        if a.as_ref().is_some_and(
          |a| b.is_some_and(
            |b| assignable(a, &b)
          )
        ) {
          Some(a.unwrap().to_owned())
        } else {
          None
        }
      },
      BinaryOperator::Assign => Some(Type::Intrinsic(intrinsics::VOID)),
      op => todo!("typeof for binop {op:#?}")
    }
  }
}

impl TypeOf for Expression {
  fn type_of(&self) -> Option<Type> {
    match self {
      Expression::Atom(atom) => Some(atom.out.clone()),
      Expression::Block(block) => block.type_of(),
      Expression::SubExpression(subexpr) => subexpr.e.type_of(),
      Expression::ControlFlow(_) => todo!("typeof for ctrlflow"),
      Expression::BinaryOperator(binop) => binop.type_of(),
      Expression::UnaryOperator(UnaryOperatorExpressionAST { out, .. }) => {
        Some(out.to_owned())
      },
    }
  }
}

// impl TypeOf for AtomExpressionAST {
//   fn type_of(&self) -> Option<Type> {
//     match &self.a {
//       AtomExpression::Literal(lit) => todo!("type_of atomexpr literal"),
//       AtomExpression::Variable(_, var_ref) => var_ref.type_of(),
//       AtomExpression::Return(_) => todo!("type_of atomexpr return"),
//       AtomExpression::Break(_) => todo!("type_of atomexpr break"),
//     }
//   }
// }

impl TypeOf for VariableReference {
  fn type_of(&self) -> Option<Type> {
    match self {
      VariableReference::ResolvedVariable(var) => {
        let var = unsafe { &**var };
        let binding_ty = var.ty.as_ref().map(|ast| Type::Defined(ast));

        println!("type_of resolved variable: {}", BlockExpressionChild::Binding(var.to_owned()).to_string());

        if binding_ty.is_some() {
          binding_ty
        } else {
          var.value
            .as_ref()
            .expect("blind binding must have type")
            .type_of()
        }
      },
      VariableReference::ResolvedArgument(arg) => {
        let arg = unsafe { &**arg };

        println!("type_of resolved_argument: {}", arg.to_string());

        Some(Type::Defined(arg))
      },
      VariableReference::ResolvedFunction(func) => {
        let func = unsafe { &**func };

        println!("type_of resolved_function: {}", func.decl.ident.to_string());

        Some(Type::Function(func))
      },
      VariableReference::ResolvedMemberOf(_parent, ident) => {
        // let parent = unsafe { &**parent };
        let ident = unsafe { &**ident };

        println!("type_of resolved_member_of: ... {}", ident.to_string());

        // let parent_ty = parent.type_of().expect("typeof of member needs to know parent type...");

        todo!("resolved member of")
      },
      VariableReference::ResolvedMemberFunction(member_func) => {
        let member_func = unsafe { &**member_func };

        println!("type_of resolved_member_function: {}", member_func.decl.decl.ident.to_string());

        None
      },
      VariableReference::ResolvedExternal(decl) => {
        let decl = unsafe { &**decl };

        Some(Type::External(decl))
      },
    }
  }
}
