use crate::tokenize::token::Span;
use crate::lang::module::Name;
use crate::lang::expr::Variable;

#[derive(Debug)]
pub struct Struct {
  pub name: Name,
  pub members: Vec<Variable>,
  pub span: Span,
}
