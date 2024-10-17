use super::{
  Compiler,
  Result,
  TakenCompilerModule,
};

pub(crate) trait Tokenize<W: CompilerWorkflow> {
  type Out;

  fn new() -> Self;
  fn tokenize(self, compiler: &mut Compiler<W>, input: TakenCompilerModule<W>) -> Result<Self::Out>;
}

pub(crate) trait Asterize<W: CompilerWorkflow> {
  type In;
  type Out;

  fn new() -> Self;
  fn asterize(self, compiler: &mut Compiler<W>, input: Self::In) -> Result<Self::Out>;
}

pub(crate) trait Translate<W: CompilerWorkflow> {
  type In;
  type Out;

  fn new() -> Self;
  fn translate(self, compiler: &mut Compiler<W>, input: Self::In) -> Result<Self::Out>;
}

pub(crate) trait Check<W: CompilerWorkflow> {
  type In;
  type Out;

  fn new() -> Self;
  fn check(self, compiler: &mut Compiler<W>, input: Self::In) -> Result<Self::Out>;
}

pub(crate) trait Generate<W: CompilerWorkflow> {
  type In;
  type Out;

  fn new() -> Self;
  fn generate(self, compiler: &mut Compiler<W>, input: Self::In) -> Result<Self::Out>;
}

pub(crate) trait Output<W: CompilerWorkflow> {
  type In;

  fn new() -> Self;
  fn output(self, compiler: &mut Compiler<W>, input: Self::In) -> Result;
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
