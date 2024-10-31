use super::*;

use std::path::{
  Path,
  PathBuf,
};

use crate::Result;
use crate::compiler::{
  CompilerJob,
  CompilerWorkflow,
  error::*,
};

/// The stored representation of a module in a CompilerStor
pub(crate) struct CompilerModule<W: CompilerWorkflow> {
  /// The path to this module's source file
  pub(crate) path: PathBuf,
  /// The current state of this module's data
  pub(crate) data: CompilerJob<W>,
}

/// The interface through which a module can be accessed after being taken from
/// the compiler store
#[derive(Debug)]
pub(crate) struct TakenCompilerModule<W: CompilerWorkflow> {
  /// Handle into the CompilerStore, for reference and reinsertion
  pub(crate) handle: CompilerStoreHandle<W>,
  /// Current data of this module
  pub(crate) data: CompilerJob<W>,
}

impl<W: CompilerWorkflow> CompilerModule<W> {
  /// Compares the path of the provided module to that of this one
  pub(crate) fn is_same_path(&self, other: &CompilerModule<W>) -> bool {
    self.path == other.path
  }
}

impl<W: CompilerWorkflow> TryFrom<&Path> for CompilerModule<W> {
  type Error = CompilerError;

  fn try_from(path: &Path) -> Result<Self> {
    if !path.exists() {
      return PathNotExistsSnafu { path }.fail();
    };

    if path.is_dir() {
      let path = path.join("index.zy");

      if path.is_dir() {
        return PathIsDirectorySnafu { path }.fail();
      };

      return path.as_path().try_into();
    }

    Ok(Self {
      data: CompilerJob::Unprocessed,
      path: path.to_path_buf(),
    })
  }
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
