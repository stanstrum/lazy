use crate::compiler::*;
use crate::tokenizer::Token;

pub(super) struct Asterizer;
pub(super) struct Translator;
pub(super) struct Checker;
pub(super) struct Generator;
pub(super) struct Outputter;

impl<W: CompilerWorkflow> Asterize<W> for Asterizer {
  type In = Vec<Token>;
  type Out = ();

  fn new() -> Self {
    todo!()
  }

  fn asterize(self, _compiler: &mut Compiler<W>, _: Self::In) -> CompilerResult<Self::Out> {
    todo!()
  }
}

impl<W: CompilerWorkflow> Translate<W> for Translator {
  type In = ();
  type Out = ();

  fn new() -> Self {
    todo!()
  }

  fn translate(self, _compiler: &mut Compiler<W>, _: Self::In) -> CompilerResult<Self::Out> {
    todo!()
  }
}

impl<W: CompilerWorkflow> Check<W> for Checker {
  type In = ();
  type Out = ();

  fn new() -> Self {
    todo!()
  }

  fn check(self, _compiler: &mut Compiler<W>, _: Self::In) -> CompilerResult<Self::Out> {
    todo!()
  }
}

impl<W: CompilerWorkflow> Generate<W> for Generator {
  type In = ();
  type Out = ();

  fn new() -> Self {
    todo!()
  }

  fn generate(self, _compiler: &mut Compiler<W>, _: Self::In) -> CompilerResult<Self::Out> {
    todo!()
  }
}

impl<W: CompilerWorkflow> Output<W> for Outputter {
  type In = ();

  fn new() -> Self {
    todo!()
  }

  fn output(self, _compiler: &mut Compiler<W>, _: Self::In) -> CompilerResult<()> {
    todo!()
  }
}
