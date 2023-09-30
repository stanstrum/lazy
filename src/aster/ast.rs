/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::HashMap;

use super::intrinsics;

pub trait GetSpan {
  fn span(&self) -> Span;
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
  pub start: usize,
  pub end: usize,
}

#[derive(Debug, Clone)]
pub struct NamespaceAST {
  pub span: Span,
  pub ident: IdentAST,
  pub map: HashMap<String, Structure>,
}

#[derive(Debug, Clone)]
pub struct KeywordAST {
  pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MemberFunctionDeclAST {
  pub span: Span,

  pub public: Option<KeywordAST>,
  pub r#static: Option<KeywordAST>,
  pub r#mut: Option<KeywordAST>,

  pub decl: FunctionDeclAST
}

#[derive(Debug, Clone)]
pub struct MemberFunctionAST {
  pub span: Span,

  pub decl: MemberFunctionDeclAST,
  pub body: BlockExpressionAST
}

#[derive(Debug, Clone)]
pub struct TraitAST {
  pub span: Span,
  pub ident: IdentAST,
  pub decls: Vec<MemberFunctionDeclAST>
}

#[derive(Debug, Clone)]
pub struct ImplAST {
  pub span: Span,

  // impl ...
  pub ty: TypeAST,
  // {
  pub methods: Vec<MemberFunctionAST>,
  // }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct TypeAliasAST {
  pub span: Span,
  pub ident: IdentAST,
  pub ty: TypeAST
}

#[derive(Debug, Clone)]
pub struct StructAST {
  pub span: Span,
  pub ident: IdentAST,
  pub members: Vec<(TypeAST, IdentAST)>
}

#[derive(Debug, Clone)]
pub enum Structure {
  Namespace(NamespaceAST),
  Function(FunctionAST),
  Struct(StructAST),
  Trait(TraitAST),
  Impl(Impl),
  TypeAlias(TypeAliasAST)
}

impl GetSpan for &Structure {
  fn span(&self) -> Span {
    match self {
      Structure::Namespace(s) => s.span(),
      Structure::Function(s) => s.span(),
      Structure::Trait(s) => s.span(),
      Structure::Impl(s) => s.span(),
      Structure::TypeAlias(s) => s.span(),
      Structure::Struct(s) => s.span()
    }
  }
}

#[derive(Debug, Clone)]
pub struct SubExpressionAST {
  pub span: Span,
  pub out: Type,
  pub e: BoxExpr
}

impl GetSpan for SubExpressionAST {
  fn span(&self) -> Span {
    GetSpan::span(&*self.e)
  }
}

type BoxExpr = Box<Expression>;

#[derive(Debug, Clone)]
pub struct UnaryOperatorExpressionAST {
  pub span: Span,
  pub out: Type,

  pub expr: BoxExpr,
  pub op: UnaryOperator
}

#[derive(Debug, Clone)]
pub struct BinaryOperatorExpressionAST {
  pub out: Type,

  pub a: BoxExpr,
  pub b: BoxExpr,

  pub op: BinaryOperator
}

impl GetSpan for BinaryOperatorExpressionAST {
  fn span(&self) -> Span {
    Span {
      start: std::cmp::min(
        self.a.span().start,
        self.b.span().start,
      ),
      end: std::cmp::max(
        self.a.span().end,
        self.b.span().end,
      ),
    }
  }
}

#[derive(Debug)]
pub enum Operator {
  UnaryPfx(UnaryPfxOperator),
  UnarySfx(UnarySfxOperator),
  Binary(BinaryOperator),
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
  UnaryPfx(UnaryPfxOperator),
  UnarySfx(UnarySfxOperator),
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryPfxOperator {
  MutRef,
  Ref,
  Deref,
  Not,
  Neg,
  NotNeg,
  BitInvert,
  PreIncrement,
  PreDecrement,
}

#[derive(Debug, Clone)]
pub enum UnarySfxOperator {
  PostIncrement,
  PostDecrement,
  Subscript { arg: BoxExpr },
  Call { args: Vec<Expression> }
}

impl PartialEq for UnarySfxOperator {
  fn eq(&self, other: &Self) -> bool {
    std::mem::discriminant(self) == std::mem::discriminant(other)
  }
}

// Last words: "I know what I'm doing."
unsafe impl Sync for UnarySfxOperator {}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
  Dot,
  DerefDot,

  Add,
  Sub,
  Mul,
  Div,
  Exp,
  Mod,

  Equals,
  NotEquals,
  Greater,
  GreaterThanEquals,
  LessThan,
  LessThanEquals,

  LogicalAnd,
  LogicalOr,
  LogicalXOR,
  BitAnd,
  BitOr,
  BitXOR,
  ArithmeticShr,
  LogicalShr,
  LogicalShl,

  // :)
  Pipe,

  Assign,

  AddAssign,
  SubAssign,
  MulAssign,
  DivAssign,
  ExpAssign,
  ModAssign,

  LogicalAndAssign,
  LogicalOrAssign,
  LogicalXORAssign,
  BitAndAssign,
  BitOrAssign,
  BitXORAssign,
  ArithmeticShrAssign,
  LogicalShrAssign,
  LogicalShlAssign,

  AssignPipe,
}

#[derive(Debug, Clone)]
pub enum Expression {
  Atom(AtomExpressionAST),
  Block(BlockExpressionAST),
  SubExpression(SubExpressionAST),
  ControlFlow(ControlFlowAST),
  BinaryOperator(BinaryOperatorExpressionAST),
  UnaryOperator(UnaryOperatorExpressionAST),
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

#[derive(Debug, Clone)]
pub enum Literal {
  UnicodeString(String),
  ByteString(String),
  CString(String),
  Char(char),
  ByteChar(char),
  NumericLiteral(String),
}

#[derive(Debug, Clone)]
pub struct LiteralAST {
  pub span: Span,
  pub l: Literal,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum ControlFlow {
  If(
    Vec<
      (Expression, BlockExpressionAST)
    >,
    Option<BlockExpressionAST>
  ),
  While(
    BoxExpr,
    Box<BlockExpressionAST>
  ),
  DoWhile(
    Box<BlockExpressionAST>,
    BoxExpr
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
pub enum VariableReference {
  Unresolved,
  ResolvedVariable(*const BindingAST),
  ResolvedArgument(*const TypeAST),
  ResolvedFunction(*const FunctionAST),
  ResolvedMemberFunction(*const MemberFunctionAST),
  ResolvedMemberOf(*const VariableReference, *const IdentAST)
}

#[derive(Debug, Clone)]
pub enum AtomExpression {
  Literal(LiteralAST),
  Variable(QualifiedAST, VariableReference),
  Return(Option<BoxExpr>),
  Break(Option<BoxExpr>),
}

#[derive(Debug, Clone)]
pub struct AtomExpressionAST {
  pub span: Span,
  pub out: Type,
  pub a: AtomExpression,
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
pub struct BindingAST {
  pub span: Span,

  pub r#mut: Option<KeywordAST>,
  pub ty: Option<TypeAST>,
  pub ident: IdentAST,
  pub value: Option<BoxExpr>
}

#[derive(Debug, Clone)]
pub enum BlockExpressionChild {
  Binding(BindingAST),
  Expression(Expression)
}

impl GetSpan for BlockExpressionChild {
  fn span(&self) -> Span {
    match self {
      BlockExpressionChild::Binding(binding) => binding.span(),
      BlockExpressionChild::Expression(expr) => expr.span(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct FunctionDeclAST {
  pub span: Span,
  pub ident: IdentAST,
  pub args: HashMap<IdentAST, TypeAST>,
  pub ret: TypeAST,
}

#[derive(Debug, Clone)]
pub struct FunctionAST {
  pub span: Span,
  pub decl: FunctionDeclAST,
  pub body: BlockExpressionAST,
}

#[derive(Debug)]
pub struct IntrinsicType {
  pub name: &'static str,
  pub bytes: usize,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum Type {
  Intrinsic(*const IntrinsicType),
  Function(*const FunctionAST),
  MemberFunction(*const MemberFunctionAST),
  Struct(*const StructAST),
  ConstReferenceTo(Box<TypeAST>),
  MutReferenceTo(Box<TypeAST>),
  ConstPtrTo(Box<TypeAST>),
  MutPtrTo(Box<TypeAST>),
  ArrayOf(Option<LiteralAST>, Box<TypeAST>),
  Defined(*const TypeAST),
  Unknown(QualifiedAST),
  UnresolvedNumeric(Literal),
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
        format!("&{}", ty.e.to_hashable())
      },
      Type::MutReferenceTo(ty) => {
        format!("&mut {}", ty.e.to_hashable())
      },
      Type::ConstPtrTo(ty) => {
        format!("*{}", ty.e.to_hashable())
      },
      Type::MutPtrTo(ty) => {
        format!("*mut {}", ty.e.to_hashable())
      },
      Type::ArrayOf(sz, ty) => {
        match sz {
          Some(sz) => {
            format!("[{}]{}", sz.to_hashable(), ty.e.to_hashable())
          },
          None => {
            format!("[]{}", ty.e.to_hashable())
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

impl std::hash::Hash for IdentAST {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.text.hash(state);
  }
}

impl std::cmp::PartialEq for IdentAST {
  fn eq(&self, other: &Self) -> bool {
    self.text == other.text
  }
}

impl std::cmp::Eq for IdentAST {}

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
  TraitAST,
  KeywordAST,
  FunctionAST,
  ControlFlowAST,
  UnaryOperatorExpressionAST,
  BindingAST,
  NamespaceAST,
  TypeAliasAST,
  StructAST,
  MemberFunctionAST
];
