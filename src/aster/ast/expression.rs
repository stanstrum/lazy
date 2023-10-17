/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use super::*;
use crate::make_get_span;

use std::collections::HashMap;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum VariableReference {
  ResolvedVariable(*const BindingAST),
  ResolvedArgument(*const TypeAST),
  ResolvedFunction(*const FunctionAST),
  #[allow(unused)]
  ResolvedMemberFunction(*const MemberFunctionAST),
  #[allow(unused)]
  ResolvedMemberOf(*const VariableReference, *const IdentAST),
  ResolvedExternal(*const ExternDeclAST)
}

#[derive(Debug, Clone)]
pub struct StructInitializerAST {
  pub span: Span,
  pub qual: QualifiedAST,
  pub members: Vec<(IdentAST, Expression)>
}

#[derive(Debug, Clone)]
pub enum AtomExpression {
  Literal(LiteralAST),
  UnresolvedVariable(QualifiedAST),
  ValueVariable(QualifiedAST, VariableReference),
  DestinationVariable(QualifiedAST, VariableReference),
  StructInitializer(StructInitializerAST),
  Return(Option<Box<Expression>>),
  #[allow(unused)]
  Break(Option<Box<Expression>>),
}

#[derive(Debug, Clone)]
pub struct AtomExpressionAST {
  pub span: Span,
  pub out: Type,
  pub a: AtomExpression,
}

#[derive(Debug, Clone)]
pub struct SubExpressionAST {
  pub span: Span,
  pub out: Type,
  pub e: Box<Expression>
}

#[derive(Debug, Clone)]
pub struct BindingAST {
  pub span: Span,

  pub r#mut: Option<KeywordAST>,
  pub ty: Option<TypeAST>,
  pub ident: IdentAST,
  pub value: Option<Box<Expression>>
}

#[derive(Debug, Clone)]
pub enum BlockExpressionChild {
  Binding(BindingAST),
  Expression(Expression)
}

#[derive(Debug, Clone)]
pub struct BlockExpressionAST {
  pub span: Span,
  pub out: Type,
  pub children: Vec<BlockExpressionChild>,
  pub vars: HashMap<IdentAST, *mut BindingAST>,
  pub returns_last: bool,
}

#[derive(Debug, Clone)]
pub enum ControlFlow {
  If(
    Vec<
      (Expression, BlockExpressionAST)
    >,
    Option<BlockExpressionAST>
  ),
  While(
    Box<Expression>,
    Box<BlockExpressionAST>
  ),
  #[allow(unused)]
  DoWhile(
    Box<BlockExpressionAST>,
    Box<Expression>
  ),
  Loop(Box<BlockExpressionAST>),
  // For(
  //   CondExpr, ElseBranch
  // ),
}

#[derive(Debug, Clone)]
pub struct ControlFlowAST {
  pub span: Span,
  pub e: ControlFlow
}

#[derive(Debug, Clone)]
pub enum Expression {
  Atom(AtomExpressionAST),
  Block(BlockExpressionAST),
  SubExpression(SubExpressionAST),
  ControlFlow(ControlFlowAST),
  UnaryOperator(UnaryOperatorExpressionAST),
  BinaryOperator(BinaryOperatorExpressionAST),
}

impl GetSpan for BlockExpressionChild {
  fn span(&self) -> Span {
    match self {
      BlockExpressionChild::Binding(binding) => binding.span(),
      BlockExpressionChild::Expression(expr) => expr.span(),
    }
  }
}

impl GetSpan for SubExpressionAST {
  fn span(&self) -> Span {
    GetSpan::span(&*self.e)
  }
}

impl GetSpan for Expression {
  fn span(&self) -> Span {
    match self {
      Expression::Atom(s) => s.span(),
      Expression::Block(s) => s.span(),
      Expression::SubExpression(s) => s.span(),
      Expression::ControlFlow(s) => s.span(),
      Expression::BinaryOperator(s) => s.span(),
      Expression::UnaryOperator(s) => s.span()
    }
  }
}

make_get_span![
  BindingAST,
  ControlFlowAST,
  UnaryOperatorExpressionAST
];
