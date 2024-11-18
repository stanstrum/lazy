pub(crate) mod error;
mod module;
mod store;
mod traits;
pub(crate) mod workflow;

use std::path::PathBuf;

pub(crate) use module::*;
pub(crate) use store::*;
pub(crate) use traits::*;

use crate::{enchant, ok, HelpSnafu, NoInputSnafu, Result};
use crate::arg_parser::CompilerOptions;

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

/// Processes the parsed command-line arguments
fn parse_compiler_options(options: CompilerOptions) -> Result<CompilerSettings> {
  let CompilerOptions {
    help,
    input_file,
    output_file,
    llc,
    cc,
    print_llvm,
  } = options;

  if help {
    return HelpSnafu.fail()?;
  };

  let Some(input_file) = input_file else {
    return NoInputSnafu.fail()?;
  };

  Ok(CompilerSettings {
    input_file,
    output_file,
    llc,
    cc,
    print_llvm,
  })
}

impl<W: CompilerWorkflow> Compiler<W> {
  /// Creates a new Compiler
  pub(crate) fn new(options: CompilerOptions) -> Result<Self> {
    let settings = parse_compiler_options(options)?;
    let output_file = std::env::current_dir().unwrap().join(&settings.output_file);

    debug!(
      "\
        Compiler initialized:\n  \
          input path: {}\n  \
          output path: {}\n  \
          llc path: {}\n  \
          cc path: {}\
          {}\
      ",
      settings.input_file.to_string_lossy(),
      output_file.to_string_lossy(),
      settings.llc.to_string_lossy(),
      settings.cc.to_string_lossy(),
      if settings.print_llvm {
        "\n  --print-llvm enabled"
      } else {
        ""
      },
    );

    Ok(Self {
      settings,
      store: CompilerStore::new(),
      context: inkwell::context::Context::create(),
    })
  }

  /// Applies compilation steps as appropriate for a certain Handle until it
  /// reaches the stage provided
  pub(crate) fn bring_to_stage(
    &mut self,
    handle: &CompilerStoreHandle<W>,
    stage: CompilationStage,
  ) -> Result {
    while {
      let module = self.store.get_module(handle);

      let Some(module_stage) = module.data.stage() else {
        warn!(
          "{}: no stage in {}",
          enchant!("bring_to_stage"),
          handle.proper_name(self)
        );
        return ok;
      };

      assert!(module_stage <= stage);
      module_stage < stage
    } {
      let mut module = self.store.take_module(handle);
      let proper_name = handle.proper_name(self);

      match module.data {
        CompilerJob::Taken => {
          warn!("{}: {proper_name}", enchant!("taken"));
          return ok;
        },
        CompilerJob::Unprocessed => {
          let input = TakenCompilerModule {
            handle: *handle,
            data: module.data,
          };

          let tokenizer = W::Tokenizer::new(input, *handle);
          let tokenized = tokenizer.tokenize(self)?;
          module.data = CompilerJob::Tokenized(tokenized);
        },
        CompilerJob::Tokenized(input) => {
          let asterizer = W::Asterizer::new(input, *handle);
          let asterized = asterizer.asterize(self)?;
          module.data = CompilerJob::Asterized(asterized);
        },
        CompilerJob::Asterized(input) => {
          let translator = W::Translator::new(input, *handle);
          let translated = translator.translate(self)?;
          module.data = CompilerJob::Translated(translated);
        },
        CompilerJob::Translated(input) => {
          debug!("{}: {proper_name}", enchant!("check"));
          let checker = W::Checker::new(input, *handle);
          let checked = checker.check(self)?;
          module.data = CompilerJob::Checked(checked);
        },
        CompilerJob::Checked(input) => {
          debug!("{}: {proper_name}", enchant!("generate"));
          let generator = W::Generator::new(input, *handle);
          let generated = generator.generate(self)?;
          module.data = CompilerJob::Generated(generated);
        },
        CompilerJob::Generated(input) => {
          debug!("{}: {proper_name}", enchant!("output"));
          let outputter = W::Outputter::new(input, *handle);
          return outputter.output(self);
        },
      };

      self.store.store_module(module);
    }

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
