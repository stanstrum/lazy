use std::rc::Rc;

use ast::Identifier;

use super::*;

/// A function argument, stored separately for organizational purposes
#[allow(unused)]
#[derive(Debug)]
pub(crate) struct FunctionArgument {
  pub(crate) parent: OpaqueParent<WeakCell<Function>>,
  /// The name of this argument
  pub(crate) name: ast::Identifier<DefaultWorkflow>,
  /// The type of this argument
  pub(crate) ty: RcCell<Type<Module>>,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Variable {
  pub(crate) name: Identifier<DefaultWorkflow>,
  pub(crate) ty: RcCell<Type<Module>>,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) enum LiteralInstructionKind {
  Integer(u64),
  Float(f64),
  String(String),
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct LiteralInstruction {
  pub(crate) kind: Rc<LiteralInstructionKind>,
  pub(crate) ty: RcCell<Type<Module>>,
  pub(crate) span: Span<DefaultWorkflow>,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct BlockInstruction {
  pub(crate) parent: OpaqueParent<WeakCell<FunctionBlock>>,
  pub(crate) variables: Vec<RcCell<Variable>>,
  pub(crate) instructions: Vec<RcCell<Instruction>>,
  pub(crate) span: Span<DefaultWorkflow>,
  pub(crate) out: RcCell<Type<Module>>,
  pub(crate) generator_id: Option<usize>,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) enum Instruction {
  Literal(LiteralInstruction),
  Block(RcCell<BlockInstruction>),
  Return {
    parent: OpaqueParent<WeakCell<FunctionBlock>>,
    value: Option<RcCell<Instruction>>,
  },
  ImplicitReturnLast {
    parent: OpaqueParent<WeakCell<FunctionBlock>>,
    block: OpaqueParent<RcCell<BlockInstruction>>,
    value: RcCell<Instruction>,
    out: RcCell<Type<Module>>,
  },
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct FunctionBlock {
  pub(crate) parent: OpaqueParent<Option<WeakCell<Function>>>,
  pub(crate) variables: Vec<RcCell<Variable>>,
  pub(crate) children: Vec<RcCell<Instruction>>,
}

/// A simple function, i.e., one that does not belong to a class or interface.
///
/// Example:
/// ```
/// main -> i32 {
///   0
/// };
/// ```
#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Function {
  pub(crate) parent: OpaqueParent<Option<WeakCell<Module>>>,
  pub(crate) name: ast::Identifier<DefaultWorkflow>,
  pub(crate) arguments: Vec<RcCell<FunctionArgument>>,
  pub(crate) body: RcCell<FunctionBlock>,
  pub(crate) return_ty: RcCell<Type<Module>>,
  pub(crate) generator_id: Option<usize>,
}
