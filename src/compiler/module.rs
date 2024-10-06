use std::path::{
  Path,
  PathBuf,
};

use super::{
  CompilerJob,
  CompilerWorkflow,
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
  type Error = String;

  fn try_from(path: &Path) -> Result<Self, Self::Error> {
    if !path.exists() {
      Err(format!("input file does not exist: {}", path.to_string_lossy()))
    } else if path.is_dir() {
      let mut path = path.to_owned();
      path.push("index.zy");

      if path.is_dir() {
        return Err(format!("input file may not be a directory: {}", path.to_string_lossy()));
      };

      CompilerModule::try_from(path.as_path())
    } else {
      Ok(Self {
        data: CompilerJob::Unprocessed,
        path: path.to_path_buf(),
      })
    }
  }
}
