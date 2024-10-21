use crate::{
  tokenizer,
  asterizer,
  todo,
};

#[derive(Clone, Copy)]
pub(crate) struct DefaultWorkflow;

impl crate::compiler::CompilerWorkflow for DefaultWorkflow {
  type Tokenizer = tokenizer::Tokenizer;
  type Asterizer = asterizer::Asterizer;
  type Translator = todo::Translator;
  type Checker = todo::Checker;
  type Generator = todo::Generator;
  type Outputter = todo::Outputter;
}
