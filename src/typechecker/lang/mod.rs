mod r#type;

use std::rc::Rc;

pub(crate) use r#type::*;

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct VariableReference {
  pub(crate) scope: Rc<Vec<Variable>>,
  pub(crate) id: usize,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) enum Value {
  Variable(VariableReference),
  Instruction(Box<Instruction>),
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) enum Instruction {
  Assign {
    dest: Value,
    value: Value,
  },
  Call {
    func: Value,
    args: Vec<Value>,
  },
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Block {
  pub(crate) variables: VariableScope,
  pub(crate) body: Vec<Instruction>,
}

impl Block {
  // pub(crate) fn new() -> Self {
  //   Self {
  //     variables: VariableScope::new(),
  //     body: vec![],
  //   }
  // }
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) enum VariableKind {
  LocalVariable,
  Argument,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Variable {
  pub(crate) kind: VariableKind,
  pub(crate) ty: Type,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct VariableScope {
  inner: Rc<Vec<Variable>>,
}

impl VariableScope {
  // pub(crate) fn new() -> Self {
  //   Self::from_vec(vec![])
  // }

  pub(crate) fn from_vec(v: Vec<Variable>) -> Self {
    Self {
      inner: Rc::new(v),
    }
  }

  pub(crate) fn get_inner(&self) -> &Rc<Vec<Variable>> {
    &self.inner
  }
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Function {
  pub(crate) arguments: VariableScope,
  pub(crate) return_ty: Type,
  pub(crate) body: Block,
}
