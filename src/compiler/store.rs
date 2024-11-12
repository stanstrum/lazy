use super::*;

use std::marker::PhantomData;

/// The progress of a compiler job
#[derive(PartialEq, PartialOrd)]
pub(crate) enum CompilationStage {
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

/// Stores modules in their respective stages of compilation
pub(crate) struct CompilerStore<W: CompilerWorkflow> {
  /// Program modules
  pub(crate) modules: Vec<CompilerModule<W>>,
  pub(super) marker: PhantomData<W>,
}

/// Represents a CompilerJob without actually taking its data.  Can be used for
/// referring to values in other modules without causing problems related to
/// circular dependencies or invalid program hierarchies
#[derive(Clone, Copy)]
pub(crate) struct CompilerStoreHandle<W: CompilerWorkflow> {
  /// The index into CompilerStore that the respective CompilerJob is stored
  pub(super) index: usize,
  pub(super) marker: PhantomData<CompilerModule<W>>,
}

impl<W: CompilerWorkflow> CompilerJob<W> {
  /// Returns the stage of this module as a CompilationStage
  pub(crate) fn stage(&self) -> Option<CompilationStage> {
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

impl<W: CompilerWorkflow> CompilerStore<W> {
  /// Creates a CompilerStore
  pub(super) fn new() -> Self {
    Self {
      modules: vec![],
      marker: Default::default(),
    }
  }

  /// Adds a new module into this store and registers it if need be
  fn add_module(&mut self, module: CompilerModule<W>) -> CompilerStoreHandle<W> {
    let index = self.modules.len();

    debug!("{}: module #{index} with path {}", crate::enchant!("add_module"), &module.path);

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
  pub(crate) fn register_module(&mut self, module: &CompilerModule<W>) -> CompilerStoreHandle<W> {
    let name = crate::enchant!("register_module");

    if let Some(handle) = self.find_module(module) {
      debug!("{name}: found already registered module #{}", handle.index);
      return handle;
    };

    let handle = self.add_module(CompilerModule {
      path: module.path.to_owned(),
      data: CompilerJob::Taken,
    });

    warn!("{name}: added unrecognized module #{}", handle.index);
    handle
  }

  /// Stores a module using the JobStore trait
  pub(crate) fn store_module<T: JobStore<W>>(&mut self, module: T) -> CompilerStoreHandle<W> {
    module.store(self)
  }

  /// Gets a reference from Self from a Handle
  pub(crate) fn get_module(&self, handle: &CompilerStoreHandle<W>) -> &CompilerModule<W> {
    &self.modules[handle.index]
  }

  /// Gets a mutable reference from Self from a Handle
  pub(crate) fn get_module_mut(&mut self, handle: &CompilerStoreHandle<W>) -> &mut CompilerModule<W> {
    &mut self.modules[handle.index]
  }

  /// Take a module from a Handle and update the internal store accordingly
  pub(crate) fn take_module(&mut self, handle: &CompilerStoreHandle<W>) -> TakenCompilerModule<W> {
    let module = self.get_module_mut(handle);

    let mut taken = TakenCompilerModule {
      handle: Clone::clone(handle),
      data: CompilerJob::Taken,
    };

    std::mem::swap(&mut module.data, &mut taken.data);

    taken
  }
}

impl<W: CompilerWorkflow> CompilerStoreHandle<W> {
  pub(crate) fn new(index: usize) -> Self {
    Self {
      index,
      marker: PhantomData,
    }
  }
}
