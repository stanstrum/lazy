use crate::compiler::*;

pub(super) struct Asterizer;
pub(super) struct Translator;
pub(super) struct Checker;
pub(super) struct Generator;
pub(super) struct Outputter;

impl<W: CompilerWorkflow> Asterize<W> for Asterizer {
  type In = ();
  type Out = ();

  fn asterize(_compiler: &mut Compiler<W>, _: Self::In) -> CompilerResult<Self::Out> {
    todo!()
  }
}

impl<W: CompilerWorkflow> Translate<W> for Translator {
  type In = ();
  type Out = ();

  fn translate(_compiler: &mut Compiler<W>, _: Self::In) -> CompilerResult<Self::Out> {
    todo!()
  }
}

impl<W: CompilerWorkflow> Check<W> for Checker {
  type In = ();
  type Out = ();

  fn check(_compiler: &mut Compiler<W>, _: Self::In) -> CompilerResult<Self::Out> {
    todo!()
  }
}

impl<W: CompilerWorkflow> Generate<W> for Generator {
  type In = ();
  type Out = ();

  fn generate(_compiler: &mut Compiler<W>, _: Self::In) -> CompilerResult<Self::Out> {
    todo!()
  }
}

impl<W: CompilerWorkflow> Output<W> for Outputter {
  type In = ();

  fn output(_compiler: &mut Compiler<W>, _: Self::In) -> CompilerResult<()> {
    todo!()
  }
}
