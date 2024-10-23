use crate::{
  tokenizer,
  asterizer,
  todo,
};

/// The default workflow for compiling Lazy code
#[derive(Clone, Copy)]
pub(crate) struct DefaultWorkflow;

impl crate::compiler::CompilerWorkflow for DefaultWorkflow {
  type Tokenizer = tokenizer::Tokenizer<Self>;
  type Asterizer = asterizer::Asterizer<Self>;
  type Translator = todo::Translator<Self>;
  type Checker = todo::Checker<Self>;
  type Generator = todo::Generator<Self>;
  type Outputter = todo::Outputter<Self>;
}
