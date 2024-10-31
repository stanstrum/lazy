use std::fmt::Debug;
use typename::TypeName;

use crate::Result;
use crate::compiler::{
  Compiler,
  CompilerStoreHandle,
  TakenCompilerModule,
};

/// The compilation step for tokenization
pub(crate) trait Tokenize<W: CompilerWorkflow>: Debug {
  type Out: Debug;

  /// Creates this tokenizer
  fn new(input: TakenCompilerModule<W>, handle: CompilerStoreHandle<W>) -> Self;
  /// Tokenizes the provided module
  fn tokenize(self, compiler: &mut Compiler<W>) -> Result<Self::Out>;
}

/// The compilation step for asterization
pub(crate) trait Asterize<W: CompilerWorkflow>: Debug {
  type In: Debug;
  type Out: Debug;

  /// Creates this asterizer
  fn new(input: Self::In, handle: CompilerStoreHandle<W>) -> Self;
  /// Asterizes the provided module
  fn asterize(self, compiler: &mut Compiler<W>) -> Result<Self::Out>;
}

/// The compilation step for translation
pub(crate) trait Translate<W: CompilerWorkflow>: Debug {
  type In: Debug;
  type Out: Debug;

  /// Creates this translator
  fn new(input: Self::In, handle: CompilerStoreHandle<W>) -> Self;
  /// Translates the provided module
  fn translate(self, compiler: &mut Compiler<W>) -> Result<Self::Out>;
}

/// The compilation step for checking
pub(crate) trait Check<W: CompilerWorkflow>: Debug {
  type In: Debug;
  type Out: Debug;

  /// Creates this checker
  fn new(input: Self::In, handle: CompilerStoreHandle<W>) -> Self;
  /// Checks the provided module
  fn check(self, compiler: &mut Compiler<W>) -> Result<Self::Out>;
}

/// The compilation step for generation
pub(crate) trait Generate<W: CompilerWorkflow>: Debug {
  type In: Debug;
  type Out: Debug;

  /// Creates this generator
  fn new(input: Self::In, handle: CompilerStoreHandle<W>) -> Self;
  /// Generates the provided module
  fn generate(self, compiler: &mut Compiler<W>) -> Result<Self::Out>;
}

/// The compilation step for outputting
pub(crate) trait Output<W: CompilerWorkflow>: Debug {
  type In: Debug;

  /// Creates this outputter
  fn new(input: Self::In, handle: CompilerStoreHandle<W>) -> Self;
  /// Outputs the provided module
  fn output(self, compiler: &mut Compiler<W>) -> Result;
}

/// The interface through which a Compiler can bring the provided modules to
/// completion
pub(crate) trait CompilerWorkflow: Debug + Clone + Copy + TypeName + Sized {
  type Tokenizer: Tokenize<Self>;
  type Asterizer: Asterize<Self, In = <Self::Tokenizer as Tokenize<Self>>::Out>;
  type Translator: Translate<Self, In = <Self::Asterizer as Asterize<Self>>::Out>;
  type Checker: Check<Self, In = <Self::Translator as Translate<Self>>::Out>;
  type Generator: Generate<Self, In = <Self::Checker as Check<Self>>::Out>;
  type Outputter: Output<Self, In = <Self::Generator as Generate<Self>>::Out>;
}
