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

impl TypeOf for Expression {
  fn type_of(&self) -> Option<Type> {
    match self {
      Expression::Atom(atom) => Some(atom.out.clone()),
      Expression::Block(block) => block.type_of(),
      Expression::SubExpression(subexpr) => subexpr.e.type_of(),
      Expression::ControlFlow(_) => todo!("typeof for ctrlflow"),
      Expression::BinaryOperator(_) => todo!("typeof for binop"),
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
      VariableReference::Unresolved => {
        println!("type_of unresolved");

        None
      },
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
    }
  }
}
