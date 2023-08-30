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
pub struct KeywordAST {
  pub span: Span,
}

#[derive(Debug)]
pub struct MemberFunctionDeclAST {
  pub span: Span,

  pub public: Option<KeywordAST>,
  pub r#static: Option<KeywordAST>,
  pub r#mut: Option<KeywordAST>,

  pub decl: FunctionDeclAST
}

#[derive(Debug)]
pub struct MemberFunctionAST {
  pub span: Span,

  pub decl: MemberFunctionDeclAST,
  pub body: BlockExpressionAST
}

#[derive(Debug)]
pub struct TraitAST {
  pub span: Span,
  pub ident: IdentAST,
  pub decls: Vec<MemberFunctionDeclAST>
}

#[derive(Debug)]
pub struct ImplAST {
  pub span: Span,

  // impl ...
  pub ty: TypeAST,
  // {
  pub methods: Vec<MemberFunctionAST>,
  // }
}

#[derive(Debug)]
pub struct ImplForAST {
  pub span: Span,

  // impl ...
  pub r#trait: QualifiedAST,
  // for ...
  pub ty: TypeAST,
  // {
  pub methods: Vec<MemberFunctionAST>
  // }
}

#[derive(Debug)]
pub enum Impl {
  Impl(ImplAST),
  ImplFor(ImplForAST)
}

impl GetSpan for Impl {
  fn span(&self) -> Span {
    match self {
      Impl::Impl(s) => &s.span,
      Impl::ImplFor(s) => &s.span,
    }.clone()
  }
}

#[derive(Debug)]
pub enum Structure {
  Namespace(NamespaceAST),
  Function(FunctionAST),
  // Struct(StructAST),
  Trait(TraitAST),
  Impl(Impl),
}

impl GetSpan for &Structure {
  fn span(&self) -> Span {
    match self {
      Structure::Namespace(s) => s.span.to_owned(),
      Structure::Function(s) => s.span.to_owned(),
      Structure::Trait(s) => s.span.to_owned(),
      Structure::Impl(s) => s.span(),
    }
  }
}

#[derive(Debug)]
pub struct Variable(pub TypeAST, pub IdentAST);

#[derive(Debug)]
pub struct SubExpressionAST {
  pub span: Span,
  pub e: Box<Expression>
}

impl GetSpan for SubExpressionAST {
  fn span(&self) -> Span {
    GetSpan::span(&*self.e)
  }
}

type BoxExpr = Box<Expression>;

#[derive(Debug)]
pub enum OperatorExpr {
  // Unary Prefix
  Ref(BoxExpr),
  Deref(BoxExpr),
  Not(BoxExpr),
  Neg(BoxExpr),
  NotNeg(BoxExpr),

  // Unary Suffix
  // ?

  // Binary
  Add(BoxExpr, BoxExpr),
  Sub(BoxExpr, BoxExpr),
  Mul(BoxExpr, BoxExpr),
  Div(BoxExpr, BoxExpr),
  Exp(BoxExpr, BoxExpr),
  Mod(BoxExpr, BoxExpr),

  Equals(BoxExpr, BoxExpr),
  NotEquals(BoxExpr, BoxExpr),
  Greater(BoxExpr, BoxExpr),
  GreaterThanEquals(BoxExpr, BoxExpr),
  LessThan(BoxExpr, BoxExpr),
  LessThanEquals(BoxExpr, BoxExpr),
  LogicalXOR(BoxExpr, BoxExpr),

  // :)
  Pipe(BoxExpr, BoxExpr),

  // sponge: use qualified for asignee
  Assign(IdentAST, BoxExpr),
  AssignPipe(IdentAST, BoxExpr),

  LogicalAnd(BoxExpr, BoxExpr),
  LogicalOr(BoxExpr, BoxExpr),
  BitAnd(BoxExpr, BoxExpr),
  BitOr(BoxExpr, BoxExpr),
  BitXor(BoxExpr, BoxExpr),
  ArithmeticShr(BoxExpr, BoxExpr),
  LogicalShr(BoxExpr, BoxExpr),
  LogicalShl(BoxExpr, BoxExpr),

  // Ternary
  Between(BoxExpr, BoxExpr, BoxExpr),
}

#[derive(Debug)]
pub enum Expression {
  Atom(AtomExpressionAST),
  Block(BlockExpressionAST),
  SubExpression(SubExpressionAST),
  ControlFlow(ControlFlowAST),
}

impl GetSpan for Expression {
  fn span(&self) -> Span {
    match self {
      Expression::Atom(s) => s.span.clone(),
      Expression::Block(s) => s.span.clone(),
      Expression::SubExpression(s) => s.span(),
      Expression::ControlFlow(s) => s.span()
    }
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
  DoWhile(
    Box<BlockExpressionAST>,
    Box<Expression>
  ),
  // For(
  //   CondExpr, ElseBranch
  // ),
}

#[derive(Debug)]
pub struct ControlFlowAST {
  pub span: Span,
  pub e: ControlFlow
}

#[derive(Debug)]
pub enum FnCallee {
  // sponge: update this to take qualified names
  // and struct members
  Qualified(QualifiedAST),
  SubExpression(SubExpressionAST)
}

#[derive(Debug)]
pub enum AtomExpression {
  Binding {
    ty: Option<TypeAST>,
    ident: IdentAST,
    value: Box<Expression>
  },
  Literal(LiteralAST),
  FnCall(Box<FnCallee>, Vec<Expression>),
  Variable(QualifiedAST),
  OperatorExpr(OperatorExpr)
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
pub struct FunctionDeclAST {
  pub span: Span,
  pub ident: IdentAST,
  pub args: Vec<Variable>,
  pub ret: TypeAST,
}

#[derive(Debug)]
pub struct FunctionAST {
  pub span: Span,
  pub decl: FunctionDeclAST,
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
  Defined(*const TypeAST),
  Unknown(QualifiedAST),
  Unresolved
}

impl TypeAST {
  pub fn to_hashable(&self) -> String {
    self.e.to_hashable()
  }
}

impl LiteralAST {
  pub fn to_hashable(&self) -> String {
    match &self.l {
      Literal::NumericLiteral(text) => text.to_owned(),
      _ => panic!("to_hashable run on non-numeric")
    }
  }
}

impl Type {
  pub fn to_hashable(&self) -> String {
    match self {
      Type::Intrinsic(s) => {
        unsafe { (**s).name }.to_owned()
      },
      Type::ConstReferenceTo(ty) => {
        format!("&{}", (**ty).e.to_hashable())
      },
      Type::MutReferenceTo(ty) => {
        format!("&mut {}", (**ty).e.to_hashable())
      },
      Type::ConstPtrTo(ty) => {
        format!("*{}", (**ty).e.to_hashable())
      },
      Type::MutPtrTo(ty) => {
        format!("*mut {}", (**ty).e.to_hashable())
      },
      Type::ArrayOf(sz, ty) => {
        match sz {
          Some(sz) => {
            format!("[{}]{}", sz.to_hashable(), (**ty).e.to_hashable())
          },
          None => {
            format!("[]{}", (**ty).e.to_hashable())
          },
        }
      },
      Type::Defined(ty) => unsafe {
        (**ty).e.to_hashable()
      },
      Type::Unknown(qual) => {
        qual.to_hashable()
      },
      _ => unimplemented!("to_hashable for {:#?}", self)
    }
  }
}

#[derive(Debug, Clone)]
pub struct TypeAST {
  pub span: Span,
  pub e: Type,
}

#[derive(Debug, Clone)]
pub struct QualifiedAST {
  pub span: Span,
  pub parts: Vec<IdentAST>,
}

impl QualifiedAST {
  pub fn to_hashable(&self) -> String {
    self.parts
      .iter()
      .map(|ident| ident.text.to_owned())
      .collect::<Vec<String>>()
      .join("::")
  }
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
  QualifiedAST,
  IdentAST,
  BlockExpressionAST,
  AtomExpressionAST,
  TypeAST,
  LiteralAST,
  ImplAST,
  ImplForAST,
  KeywordAST,
  FunctionAST,
  ControlFlowAST
];
