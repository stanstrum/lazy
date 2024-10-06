use super::{
  Compiler,
  CompilerResult,
  TakenCompilerModule,
};

pub(crate) trait Tokenize<W: CompilerWorkflow> {
  type Out;

  fn tokenize(compiler: &mut Compiler<W>, input: TakenCompilerModule<W>) -> CompilerResult<Self::Out>;
}

pub(crate) trait Asterize<W: CompilerWorkflow> {
  type In;
  type Out;

  fn asterize(compiler: &mut Compiler<W>, input: Self::In) -> CompilerResult<Self::Out>;
}

pub(crate) trait Translate<W: CompilerWorkflow> {
  type In;
  type Out;

  fn translate(compiler: &mut Compiler<W>, input: Self::In) -> CompilerResult<Self::Out>;
}

pub(crate) trait Check<W: CompilerWorkflow> {
  type In;
  type Out;

  fn check(compiler: &mut Compiler<W>, input: Self::In) -> CompilerResult<Self::Out>;
}

pub(crate) trait Generate<W: CompilerWorkflow> {
  type In;
  type Out;

  fn generate(compiler: &mut Compiler<W>, input: Self::In) -> CompilerResult<Self::Out>;
}

pub(crate) trait Output<W: CompilerWorkflow> {
  type In;

  fn output(compiler: &mut Compiler<W>, input: Self::In) -> CompilerResult<()>;
}

pub(crate) trait CompilerWorkflow
  where Self: Sized + Clone + Copy
{
  type Tokenizer: Tokenize<Self>;
  type Asterizer: Asterize<Self, In = <Self::Tokenizer as Tokenize<Self>>::Out>;
  type Translator: Translate<Self, In = <Self::Asterizer as Asterize<Self>>::Out>;
  type Checker: Check<Self, In = <Self::Translator as Translate<Self>>::Out>;
  type Generator: Generate<Self, In = <Self::Checker as Check<Self>>::Out>;
  type Outputter: Output<Self, In = <Self::Generator as Generate<Self>>::Out>;
}
