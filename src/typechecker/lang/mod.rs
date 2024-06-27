mod r#type;

pub(crate) use r#type::*;

#[derive(Debug)]
enum Instruction {}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Variable {
  ty: Type,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Block {
  variables: Vec<Variable>,
  body: Vec<Instruction>,
}

impl Block {
  pub(crate) fn new() -> Self {
    Self {
      variables: vec![],
      body: vec![],
    }
  }
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Function {
  pub(crate) arguments: Vec<Type>,
  pub(crate) return_ty: Type,
  pub(crate) body: Block,
}
