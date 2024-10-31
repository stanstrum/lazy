use typename::TypeName;

use crate::{
  pipeline::*,
  todo,
};

/// The default workflow for compiling Lazy code
#[derive(Debug, Clone, Copy, TypeName)]
pub(crate) struct DefaultWorkflow;

impl crate::compiler::CompilerWorkflow for DefaultWorkflow {
  type Tokenizer = tokenizer::Tokenizer<Self>;
  type Asterizer = asterizer::Asterizer<Self>;
  type Translator = translator::Translator<Self>;
  type Checker = checker::Checker<Self>;
  type Generator = todo::Generator<Self>;
  type Outputter = todo::Outputter<Self>;
}
