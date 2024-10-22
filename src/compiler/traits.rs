use crate::Result;
use crate::compiler::{
  Compiler,
  TakenCompilerModule,
};

/// The compilation step for tokenization
pub(crate) trait Tokenize<W: CompilerWorkflow> {
  type Out;

  /// Creates this tokenizer
  fn new() -> Self;
  /// Tokenizes the provided module
  fn tokenize(self, compiler: &mut Compiler<W>, input: TakenCompilerModule<W>) -> Result<Self::Out>;
}

/// The compilation step for asterization
pub(crate) trait Asterize<W: CompilerWorkflow> {
  type In;
  type Out;

  /// Creates this asterizer
  fn new() -> Self;
  /// Asterizes the provided module
  fn asterize(self, compiler: &mut Compiler<W>, input: Self::In) -> Result<Self::Out>;
}

/// The compilation step for translation
pub(crate) trait Translate<W: CompilerWorkflow> {
  type In;
  type Out;

  /// Creates this translator
  fn new() -> Self;
  /// Translates the provided module
  fn translate(self, compiler: &mut Compiler<W>, input: Self::In) -> Result<Self::Out>;
}

/// The compilation step for checking
pub(crate) trait Check<W: CompilerWorkflow> {
  type In;
  type Out;

  /// Creates this checker
  fn new() -> Self;
  /// Checks the provided module
  fn check(self, compiler: &mut Compiler<W>, input: Self::In) -> Result<Self::Out>;
}

/// The compilation step for generation
pub(crate) trait Generate<W: CompilerWorkflow> {
  type In;
  type Out;

  /// Creates this generator
  fn new() -> Self;
  /// Generates the provided module
  fn generate(self, compiler: &mut Compiler<W>, input: Self::In) -> Result<Self::Out>;
}

/// The compilation step for outputting
pub(crate) trait Output<W: CompilerWorkflow> {
  type In;

  /// Creates this outputter
  fn new() -> Self;
  /// Outputs the provided module
  fn output(self, compiler: &mut Compiler<W>, input: Self::In) -> Result;
}

/// The interface through which a Compiler can bring the provided modules to
/// completion
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
