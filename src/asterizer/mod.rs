use crate::Result;
use crate::tokenizer::Token;
use crate::compiler::{
  CompilerWorkflow,
  Asterize,
  Compiler,
};

pub(super) struct Asterizer;

impl<W: CompilerWorkflow> Asterize<W> for Asterizer {
  type In = Vec<Token>;
  type Out = ();

  fn new() -> Self {
    Self
  }

  fn asterize(self, compiler: &mut Compiler<W>, input: Self::In) -> Result<Self::Out> {
    todo!()
  }
}
