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
  pub(crate) scope: Rc<RefCell<VariableScope>>,
  pub(crate) id: usize,
  pub(crate) span: Span,
}

impl VariableReference {
  pub(crate) fn get(&self) -> Variable {
    self.scope.borrow().inner.get(self.id).unwrap().to_owned()
  }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub(crate) enum Value {
  Variable(VariableReference),
  Instruction(Box<Instruction>),
  Literal {
    literal: tokenizer::Literal,
    ty: TypeCell,
  },
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
  Return {
    value: Option<Value>,
    to: TypeCell,
    span: Span,
  },
  Value(Value),
  Block(Block),
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub(crate) struct Block {
  pub(crate) variables: Rc<RefCell<VariableScope>>,
  pub(crate) body: Vec<Instruction>,
  pub(crate) span: Span,
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
  pub(crate) inner: Vec<Variable>,
  pub(crate) generator_id: Option<usize>,
}

impl GetSpan for Variable {
  fn get_span(&self) -> Span {
    self.span
  }
}

impl GetSpan for Instruction {
  fn get_span(&self) -> Span {
    match self {
      Instruction::Value(value) => value.get_span(),
      | Instruction::Assign { span, .. }
      | Instruction::Call { span, .. }
      | Instruction::Return { span, .. } => *span,
      Instruction::Block(block) => block.get_span(),
    }
  }
}

impl VariableScope {
  pub(crate) fn from_vec(inner: Vec<Variable>) -> Self {
    Self {
      inner,
      generator_id: None,
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

impl GetSpan for Block {
  fn get_span(&self) -> Span {
    self.span
  }
}

impl GetSpan for Value {
  fn get_span(&self) -> Span {
    match self {
      Value::Variable(var) => var.get().get_span(),
      Value::Instruction(inst) => inst.get_span(),
      Value::Literal { literal, .. } => literal.get_span(),
    }
  }
}

impl GetSpan for Function {
  fn get_span(&self) -> Span {
    self.span
  }
}

impl GetSpan for VariableReference {
  fn get_span(&self) -> Span {
    self.span
  }
}
