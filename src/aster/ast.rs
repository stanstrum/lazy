/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::HashMap;

pub trait GetSpan {
  fn span(&self) -> Span;
}

#[derive(Debug, Clone)]
pub struct Span {
  pub start: usize,
  pub end: usize,
}

#[derive(Debug)]
pub struct NamespaceAST {
  pub span: Span,
  pub ident: IdentAST,
  pub map: HashMap<String, Structure>,
}

#[derive(Debug)]
pub enum Structure {
  NamespaceAST(NamespaceAST),
  FunctionAST(FunctionAST),
}

impl GetSpan for &Structure {
  fn span(&self) -> Span {
    match self {
      Structure::NamespaceAST(s) => &s.span,
      Structure::FunctionAST(s) => &s.span,
    }.clone()
  }
}

#[derive(Debug)]
pub struct Variable(pub TypeAST, pub IdentAST);

#[derive(Debug)]
pub enum Expression {
  Atom(AtomExpressionAST),
  Block(BlockExpressionAST),
}

impl GetSpan for &Expression {
  fn span(&self) -> Span {
    match self {
      Expression::Atom(s) => &s.span,
      Expression::Block(s) => &s.span,
    }.clone()
  }
}

#[derive(Debug, Clone)]
pub enum Literal {
  String(String),
  ByteString(String),
  Char,
  ByteChar,
  NumericLiteral(String),
}

#[derive(Debug, Clone)]
pub struct LiteralAST {
  pub span: Span,
  pub l: Literal,
}

#[derive(Debug)]
pub struct CondExpr(Expression, Expression);

#[derive(Debug)]
pub struct ElseBranch(Option<Expression>);

#[derive(Debug)]
pub enum ControlFlow {
  If(Vec<CondExpr>, ElseBranch),
  While(CondExpr),
  DoWhile(CondExpr),
  For(CondExpr, ElseBranch),
}

#[derive(Debug)]
pub enum AtomExpression {
  Assignment {
    ty: Option<TypeAST>,
    ident: IdentAST,
    value: Box<Expression>
  },
  Literal(LiteralAST)
}

#[derive(Debug)]
pub struct AtomExpressionAST {
  pub span: Span,
  pub a: AtomExpression,
  pub out: Type,
}

#[derive(Debug)]
pub struct BlockExpressionAST {
  pub span: Span,
  pub children: Vec<Expression>,
  pub returns_last: bool,
  pub out: Type,
}

#[derive(Debug)]
pub struct FunctionAST {
  pub span: Span,
  pub ident: IdentAST,
  pub args: Vec<Variable>,
  pub ret: TypeAST,
  pub body: BlockExpressionAST,
}

pub struct IntrinsicType {
  pub name: &'static str,
  pub bytes: usize,
}

#[derive(Debug, Clone)]
pub enum Type {
  Intrinsic(*const IntrinsicType),
  ConstReferenceTo(Box<TypeAST>),
  MutReferenceTo(Box<TypeAST>),
  ConstPtrTo(Box<TypeAST>),
  MutPtrTo(Box<TypeAST>),
  ArrayOf(Option<LiteralAST>, Box<TypeAST>),
  Defined(*mut TypeAST),
  Unknown(IdentAST),
  Unresolved,
}

#[derive(Debug, Clone)]
pub struct TypeAST {
  pub span: Span,
  pub e: Type,
}

#[derive(Debug, Clone)]
pub struct IdentAST {
  pub span: Span,
  pub text: String,
}

macro_rules! make_get_span [
  ($i:ident) => {
    impl GetSpan for $i {
      fn span(&self) -> Span {
        self.span.clone()
      }
    }
  };

  ($first:ident, $($rest:ident),+) => {
    make_get_span!($first);
    make_get_span!($($rest),+);
  };
];

make_get_span![
  IdentAST,
  BlockExpressionAST,
  AtomExpressionAST,
  TypeAST,
  LiteralAST
];
