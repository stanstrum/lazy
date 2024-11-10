mod module;
mod traits;
mod store;
pub(crate) mod workflow;
pub(crate) mod error;

pub(crate) use module::*;
pub(crate) use traits::*;
pub(crate) use store::*;

use crate::{Result, ok};
use std::path::PathBuf;

/// Parsed CompilerOptions after default values and IO checks
#[allow(unused)]
pub(super) struct CompilerSettings {
  /// Program entry point
  pub(super) input_file: PathBuf,
  /// Output executable path
  pub(super) output_file: PathBuf,
  /// Path of LLC executable
  pub(super) llc: PathBuf,
  /// Path of CC executable
  pub(super) cc: PathBuf,
  /// Print LLVM code during generation
  pub(super) print_llvm: bool,
}

/// The Lazy compiler
pub(super) struct Compiler<W: CompilerWorkflow> {
  /// Settings parsed from options
  pub(crate) settings: CompilerSettings,
  /// Module store
  pub(crate) store: CompilerStore<W>,
  pub(crate) context: inkwell::context::Context,
}

impl<W: CompilerWorkflow> Compiler<W> {
  /// Creates a new Compiler
  pub(crate) fn new(settings: CompilerSettings) -> Self {
    info!(
      "\
        Compiler initialized:\n  \
          Input path: {:?}\n  \
          Output path: {:?}\n  \
          LLC path: {:?}\n  \
          CC path: {:?}\n  \
          Print LLVM: {:?}\
      ",
      &settings.input_file,
      &settings.output_file,
      &settings.llc,
      &settings.cc,
      &settings.print_llvm,
    );

    Self {
      settings,
      store: CompilerStore::new(),
      context: inkwell::context::Context::create(),
    }
  }

  /// Applies compilation steps as appropriate for a certain Handle until it
  /// reaches the stage provided
  pub(crate) fn bring_to_stage(&mut self, handle: &CompilerStoreHandle<W>, stage: CompilationStage) -> Result {
    while {
      let module = self.store.get_module(handle);

      let Some(module_stage) = module.data.stage() else {
        warn!("module {} (id {}): no stage", module.path.to_string(), handle.index);
        return ok;
      };

      assert!(module_stage <= stage);
      module_stage < stage
    } {
      let mut module = self.store.take_module(handle);
      let log_prefix = || format!("module {:?} (id #{})", &self.store.get_module(handle).path, handle.index);

      match module.data {
        CompilerJob::Taken => {
          warn!("{}: taken", log_prefix());
          return ok;
        },
        CompilerJob::Unprocessed => {
          info!("{}: tokenize", log_prefix());
          let input = TakenCompilerModule {
            handle: *handle,
            data: module.data,
          };

          let tokenizer = W::Tokenizer::new(input, *handle);
          let tokenized = tokenizer.tokenize(self)?;
          module.data = CompilerJob::Tokenized(tokenized);
        },
        CompilerJob::Tokenized(input) => {
          info!("{}: asterize", log_prefix());
          let asterizer = W::Asterizer::new(input, *handle);
          let asterized = asterizer.asterize(self)?;
          module.data = CompilerJob::Asterized(asterized);
        },
        CompilerJob::Asterized(input) => {
          info!("{}: translate", log_prefix());
          let translator = W::Translator::new(input, *handle);
          let translated = translator.translate(self)?;
          module.data = CompilerJob::Translated(translated);
        },
        CompilerJob::Translated(input) => {
          info!("{}: check", log_prefix());
          let checker = W::Checker::new(input, *handle);
          let checked = checker.check(self)?;
          module.data = CompilerJob::Checked(checked);
        },
        CompilerJob::Checked(input) => {
          info!("{}: generate", log_prefix());
          let generator = W::Generator::new(input, *handle);
          let generated = generator.generate(self)?;
          module.data = CompilerJob::Generated(generated);
        },
        CompilerJob::Generated(input) => {
          info!("{}: output", log_prefix());
          let outputter = W::Outputter::new(input, *handle);
          return outputter.output(self);
        },
      };

      self.store.store_module(module);
    };

    ok
  }

  /// Compile the program as configured via the provided settings
  pub(crate) fn compile(&mut self) -> Result {
    let module: CompilerModule<W> = self.settings.input_file.as_path().try_into()?;
    let handle = self.store.store_module(module);

    self.bring_to_stage(&handle, CompilationStage::Done)?;

    ok
  }
}

impl<W: CompilerWorkflow + std::fmt::Debug> std::fmt::Debug for CompilerStoreHandle<W> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_fmt(format_args!("CompilerStoreHandle({})", self.index))
  }
}
