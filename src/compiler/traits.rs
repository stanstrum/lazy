use std::fmt::Debug;

use typename::TypeName;

use super::*;
use crate::compiler::{Compiler, CompilerStoreHandle, TakenCompilerModule};
use crate::Result;

/// Insertion into a CompilerStore
pub(crate) trait JobStore<W: CompilerWorkflow>
where
  Self: Sized,
{
  /// Stores this job's data via handle
  fn store_by_handle(
    self,
    store: &mut CompilerStore<W>,
    handle: CompilerStoreHandle<W>,
  ) -> CompilerStoreHandle<W>;
  /// Stores this job's data via owned data
  fn store(self, store: &mut CompilerStore<W>) -> CompilerStoreHandle<W>;
}

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

/// A file in the process of being compiled
#[allow(unused)]
#[derive(Debug)]
pub(crate) enum CompilerJob<W: CompilerWorkflow> {
  /// Has been taken by a compilation step and is therefore unavailable
  Taken,
  /// Has not been processed yet
  Unprocessed,
  /// Has been tokenized
  Tokenized(<W::Tokenizer as Tokenize<W>>::Out),
  /// Has been asterized
  Asterized(<W::Asterizer as Asterize<W>>::Out),
  /// Has been translated
  Translated(<W::Translator as Translate<W>>::Out),
  /// Has been checked
  Checked(<W::Checker as Check<W>>::Out),
  /// Has been generated
  Generated(<W::Generator as Generate<W>>::Out),
}
