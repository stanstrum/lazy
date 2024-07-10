mod r#type;
pub(super) mod pretty_print;

use std::rc::Rc;
use std::cell::RefCell;

use crate::tokenizer;
use crate::tokenizer::{
  Span,
  GetSpan
};

pub(crate) use r#type::*;

#[allow(unused)]
#[derive(Debug, Clone)]
pub(crate) struct VariableReference {
  pub(crate) scope: Rc<RefCell<Vec<Variable>>>,
  pub(crate) id: usize,
  pub(crate) span: Span,
}

impl VariableReference {
  pub(crate) fn get(&self) -> Variable {
    self.scope.borrow().get(self.id).unwrap().to_owned()
  }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub(crate) enum Value {
  Variable(VariableReference),
  Instruction(Box<Instruction>),
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub(crate) enum Instruction {
  Assign {
    dest: Value,
    value: Value,
    span: Span,
  },
  Call {
    func: Value,
    args: Vec<Value>,
    span: Span,
  },
  Literal(tokenizer::Literal),
  Return {
    value: Value,
    span: Span,
  },
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Block {
  pub(crate) variables: VariableScope,
  pub(crate) body: Vec<Instruction>,
  pub(crate) span: Span,
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
#[derive(Debug, Clone)]
pub(crate) enum VariableKind {
  LocalVariable,
  Argument,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub(crate) struct Variable {
  pub(crate) name: String,
  pub(crate) kind: VariableKind,
  pub(crate) ty: TypeCell,
  pub(crate) span: Span,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct VariableScope {
  pub(crate) inner: Rc<RefCell<Vec<Variable>>>,
}

impl GetSpan for Variable {
  fn get_span(&self) -> Span {
    self.span
  }
}

impl GetSpan for Instruction {
  fn get_span(&self) -> Span {
    match self {
      Instruction::Literal(literal) => literal.get_span(),
      | Instruction::Assign { span, .. }
      | Instruction::Call { span, .. }
      | Instruction::Return { span, .. } => *span,
    }
  }
}

impl VariableScope {
  // pub(crate) fn new() -> Self {
  //   Self::from_vec(vec![])
  // }

  pub(crate) fn from_vec(v: Vec<Variable>) -> Self {
    Self {
      inner: Rc::new(RefCell::new(v)),
    }
  }
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Function {
  pub(crate) name: String,
  pub(crate) arguments: VariableScope,
  pub(crate) return_ty: TypeCell,
  pub(crate) body: Block,
  pub(crate) span: Span,
}

impl GetSpan for Function {
  fn get_span(&self) -> Span {
    self.span
  }
}
