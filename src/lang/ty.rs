use crate::tokenize::token::Span;
use crate::string_pool::PoolId;

#[derive(Debug)]
pub struct Qualified {
  pub parts: Vec<PoolId>,
  pub span: Span,
}

#[derive(Debug)]
pub enum Type {
  Unresolved(Qualified),
}
