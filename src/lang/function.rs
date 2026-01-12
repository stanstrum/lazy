use crate::lang::ty::Type;
use crate::tokenize::token::Span;
use crate::string_pool::PoolId;

#[derive(Debug)]
pub struct FunctionArgument {
  pub name: PoolId,
  pub ty: Type,
  pub span: Span,
}

#[derive(Debug)]
pub struct FunctionHeader {
  pub name: PoolId,
  pub ret_ty: Option<Type>,
  pub arguments: Vec<FunctionArgument>,
  pub span: Span,
}

#[derive(Debug)]
pub struct Function {
  pub header: FunctionHeader,
  pub span: Span,
}
