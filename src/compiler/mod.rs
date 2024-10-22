mod module;
mod traits;
pub(crate) mod workflow;
pub(crate) mod error;

pub(crate) use module::CompilerModule;
pub(crate) use traits::*;

use crate::{Result, ok};
use std::path::PathBuf;
use std::marker::PhantomData;

/// A file in the process of being compiled
#[allow(unused)]
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

/// Stores modules in their respective stages of compilation
pub(crate) struct CompilerStore<W: CompilerWorkflow> {
  /// Program modules
  modules: Vec<CompilerModule<W>>,
  marker: PhantomData<W>,
}

/// Represents a CompilerJob without actually taking its data.  Can be used for
/// referring to values in other modules without causing problems related to
/// circular dependencies or invalid program hierarchies
#[derive(Clone, Copy)]
pub(crate) struct CompilerStoreHandle<W: CompilerWorkflow> {
  /// The index into CompilerStore that the respective CompilerJob is stored
  index: usize,
  marker: PhantomData<CompilerModule<W>>,
}

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
}

/// The Lazy compiler
pub(super) struct Compiler<W: CompilerWorkflow> {
  /// Settings parsed from options
  pub(crate) settings: CompilerSettings,
  /// Module store
  pub(crate) store: CompilerStore<W>,
}

impl<W: CompilerWorkflow> CompilerJob<W> {
  /// Returns the stage of this module as a CompilationStage
  fn stage(&self) -> Option<CompilationStage> {
    match self {
      CompilerJob::Taken => None,
      CompilerJob::Unprocessed => Some(CompilationStage::Tokenize),
      CompilerJob::Tokenized(_) => Some(CompilationStage::Asterize),
      CompilerJob::Asterized(_) => Some(CompilationStage::Translate),
      CompilerJob::Translated(_) => Some(CompilationStage::Check),
      CompilerJob::Checked(_) => Some(CompilationStage::Generate),
      CompilerJob::Generated(_) => Some(CompilationStage::Output),
    }
  }
}

/// The progress of a compiler job
#[derive(PartialEq, PartialOrd)]
enum CompilationStage {
  /// Needs to be tokenized
  Tokenize,
  /// Needs to be asterized
  Asterize,
  /// Needs to be translated
  Translate,
  /// Needs to be checked
  Check,
  /// Needs to be generated
  Generate,
  /// Needs to be outputted
  Output,
  /// All done
  Done,
}

/// The interface through which a module can be accessed after being taken from
/// the compiler store
pub(crate) struct TakenCompilerModule<W: CompilerWorkflow> {
  /// Handle into the CompilerStore, for reference and reinsertion
  pub(crate) handle: CompilerStoreHandle<W>,
  /// Current data of this module
  pub(crate) data: CompilerJob<W>,
}

/// Insertion into a CompilerStore
trait JobStore<W: CompilerWorkflow> where Self: Sized {
  /// Stores this job's data via handle
  fn store_by_handle(self, store: &mut CompilerStore<W>, handle: CompilerStoreHandle<W>) -> CompilerStoreHandle<W>;
  /// Stores this job's data via owned data
  fn store(self, store: &mut CompilerStore<W>) -> CompilerStoreHandle<W>;
}

impl<W: CompilerWorkflow> JobStore<W> for CompilerModule<W> {
  fn store_by_handle(self, store: &mut CompilerStore<W>, handle: CompilerStoreHandle<W>) -> CompilerStoreHandle<W> {
    store.modules.insert(handle.index, self);
    handle
  }

  fn store(self, store: &mut CompilerStore<W>) -> CompilerStoreHandle<W> {
    let handle = store.register_module(&self);
    self.store_by_handle(store, handle)
  }
}

impl<W: CompilerWorkflow> JobStore<W> for TakenCompilerModule<W> {
  fn store_by_handle(self, store: &mut CompilerStore<W>, handle: CompilerStoreHandle<W>) -> CompilerStoreHandle<W> {
    store.modules[handle.index].data = self.data;
    handle
  }

  fn store(self, store: &mut CompilerStore<W>) -> CompilerStoreHandle<W> {
    let handle = self.handle;
    self.store_by_handle(store, handle)
  }
}

impl<W: CompilerWorkflow> CompilerStore<W> {
  /// Creates a CompilerStore
  fn new() -> Self {
    Self {
      modules: vec![],
      marker: Default::default(),
    }
  }

  /// Adds a new module into this store and registers it if need be
  fn add_module(&mut self, module: CompilerModule<W>) -> CompilerStoreHandle<W> {
    let index = self.modules.len();

    self.modules.push(module);

    CompilerStoreHandle {
      index,
      marker: Default::default(),
    }
  }

  /// Finds a module's handle if it exists
  fn find_module(&self, module: &CompilerModule<W>) -> Option<CompilerStoreHandle<W>> {
    for (index, curr) in self.modules.iter().enumerate() {
      if module.is_same_path(curr) {
        return Some(CompilerStoreHandle {
          index,
          marker: Default::default(),
        });
      };
    };

    None
  }

  /// Creates a new entry into Self if the provided module is not already stored
  fn register_module(&mut self, module: &CompilerModule<W>) -> CompilerStoreHandle<W> {
    if let Some(handle) = self.find_module(module) {
      return handle;
    };

    self.add_module(CompilerModule {
      path: module.path.to_owned(),
      data: CompilerJob::Taken,
    })
  }

  /// Stores a module using the JobStore trait
  fn store_module<T: JobStore<W>>(&mut self, module: T) -> CompilerStoreHandle<W> {
    module.store(self)
  }

  /// Gets a reference from Self from a Handle
  pub(crate) fn get_module(&self, handle: &CompilerStoreHandle<W>) -> &CompilerModule<W> {
    &self.modules[handle.index]
  }

  /// Gets a mutable reference from Self from a Handle
  fn get_module_mut(&mut self, handle: &CompilerStoreHandle<W>) -> &mut CompilerModule<W> {
    &mut self.modules[handle.index]
  }

  /// Take a module from a Handle and update the internal store accordingly
  fn take_module(&mut self, handle: &CompilerStoreHandle<W>) -> TakenCompilerModule<W>  {
    let module = self.get_module_mut(handle);

    let mut taken = TakenCompilerModule::<W> {
      handle: <CompilerStoreHandle<W> as Clone>::clone(handle),
      data: CompilerJob::Taken::<W>,
    };

    std::mem::swap(&mut module.data, &mut taken.data);

    taken
  }
}

impl<W: CompilerWorkflow> Compiler<W> {
  /// Creates a new Compiler
  pub(crate) fn new(settings: CompilerSettings) -> Self {
    Self {
      settings,
      store: CompilerStore::new(),
    }
  }

  /// Applies compilation steps as appropriate for a certain Handle until it
  /// reaches the stage provided
  fn bring_to_stage(&mut self, handle: &CompilerStoreHandle<W>, stage: CompilationStage) -> Result {
    while {
      let module = self.store.get_module(handle);

      let Some(module_stage) = module.data.stage() else {
        warn!("module {} (id {}): no stage", module.path.to_string_lossy(), handle.index);
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
          trace!("{}: tokenize", log_prefix());
          let tokenized = W::Tokenizer::new().tokenize(self, TakenCompilerModule {
            handle: *handle,
            data: module.data,
          })?;
          module.data = CompilerJob::Tokenized(tokenized);
        },
        CompilerJob::Tokenized(input) => {
          trace!("{}: asterize", log_prefix());
          let asterized = W::Asterizer::new().asterize(self, input)?;
          module.data = CompilerJob::Asterized(asterized);
        },
        CompilerJob::Asterized(input) => {
          trace!("{}: translate", log_prefix());
          let translated = W::Translator::new().translate(self, input)?;
          module.data = CompilerJob::Translated(translated);
        },
        CompilerJob::Translated(input) => {
          trace!("{}: check", log_prefix());
          let checked = W::Checker::new().check(self, input)?;
          module.data = CompilerJob::Checked(checked);
        },
        CompilerJob::Checked(input) => {
          trace!("{}: generate", log_prefix());
          let generated = W::Generator::new().generate(self, input)?;
          module.data = CompilerJob::Generated(generated);
        },
        CompilerJob::Generated(input) => {
          trace!("{}: output", log_prefix());
          W::Outputter::new().output(self, input)?;

          return ok;
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
