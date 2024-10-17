use std::path::{
  Path,
  PathBuf,
};

use super::{
  CompilerJob,
  CompilerWorkflow,
  Result,
  error::*,
};

pub(crate) struct CompilerModule<W: CompilerWorkflow> {
  pub(crate) path: PathBuf,
  pub(crate) data: CompilerJob<W>,
}

impl<W: CompilerWorkflow> CompilerModule<W> {
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
