use std::collections::HashMap;

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

#[derive(Debug)]
pub struct Variable(pub TypeAST, pub IdentAST);

#[derive(Debug)]
pub enum Expression {
  Atom(AtomExpressionAST),
  Block(BlockExpressionAST),
}

#[derive(Debug, Clone)]
pub enum Literal {
  String(String),
  ByteString,
  Char,
  ByteChar,
  NumericLiteral,
  SuffixedNumericLiteral,
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
  Literal(Literal)
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
  ArrayOf(Option<u32>, Box<TypeAST>),
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
