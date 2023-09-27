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

impl TypeOf for Expression {
  fn type_of(&self) -> Option<Type> {
    match self {
      Expression::Atom(atom) => atom.type_of(),
      Expression::Block(block) => {
        if block.returns_last {
          match block.children.last().unwrap() {
            BlockExpressionChild::Binding(_) => panic!("returns last but last is a binding..."),
            BlockExpressionChild::Expression(expr) => expr.type_of(),
          }
        } else {
          Some(Type::Intrinsic(&intrinsics::VOID))
        }
      },
      Expression::SubExpression(subexpr) => subexpr.e.type_of(),
      Expression::ControlFlow(_) => todo!("typeof for ctrlflow"),
      Expression::BinaryOperator(_) => todo!("typeof for binop"),
      Expression::UnaryOperator(_) => todo!("typeof for unaryop"),
    }
  }
}

impl TypeOf for AtomExpressionAST {
  fn type_of(&self) -> Option<Type> {
    match &self.a {
      AtomExpression::Literal(_) => todo!("type_of atomexpr literal"),
      AtomExpression::Variable(_, var_ref) => var_ref.type_of(),
      AtomExpression::Return(_) => todo!("type_of atomexpr return"),
      AtomExpression::Break(_) => todo!("type_of atomexpr break"),
    }
  }
}

impl TypeOf for VariableReference {
  fn type_of(&self) -> Option<Type> {
    match self {
      VariableReference::Unresolved => None,
      VariableReference::ResolvedVariable(var) => {
        let var = unsafe { &**var };
        let ty = var.ty.as_ref().map(|ast| Type::Defined(ast));

        ty
      },
      VariableReference::ResolvedArgument(arg) => {
        let arg = unsafe { &**arg };

        Some(Type::Defined(arg))
      },
      VariableReference::ResolvedFunction(func) => {
        let func = unsafe { &**func };

        Some(Type::Function(func))
      },
      VariableReference::ResolvedMemberOf(parent, ident) => {
        let parent = unsafe { &**parent };
        let ident = unsafe { &**ident };

        let parent_ty = parent.type_of().expect("typeof of member needs to know parent type...");

        todo!("resolved member of")
      },
      VariableReference::ResolvedMemberFunction(member_func) => {
        let member_func = unsafe { &**member_func };

        Some(Type::MemberFunction(member_func))
      },
    }
  }
}
