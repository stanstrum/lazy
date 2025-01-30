use std::{io::Read, path::Path};

use include_directory::{include_directory, Dir, DirEntry};
use snafu::{whatever, Whatever};
use utf8_read::Char;

use super::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(super) enum FileKind {
  EmbeddedStdSource,
  SourceFile,
}

#[derive(Debug, PartialEq, Clone)]
pub(super) struct LazyFile {
  pub kind: FileKind,
  pub path: PathBuf,
}

fn get_stl_source(path: &Path) -> Option<&'static [u8]> {
  static STANDARD_LIBRARY: Dir<'_> = include_directory!("$CARGO_MANIFEST_DIR/std");

  match STANDARD_LIBRARY.get_entry(path)? {
    DirEntry::File(file) => return Some(file.contents()),
    DirEntry::Dir(dir) => dir.get_file("index.zy").map(|file| file.contents()),
  }
}

impl LazyFile {
  pub(super) fn standard_library() -> Self {
    Self {
      kind: FileKind::EmbeddedStdSource,
      path: PathBuf::from("/"),
    }
  }

  pub(super) fn new(path: PathBuf) -> Self {
    Self {
      kind: FileKind::SourceFile,
      path,
    }
  }

  pub(super) fn solidify(&mut self) -> Result<(), Whatever> {
    match &self.kind {
      FileKind::EmbeddedStdSource => todo!(),
      FileKind::SourceFile => {
        if self.path.is_file() {
          return Ok(());
        };

        if self.path.is_dir() {
          self.path = self.path.join("index.zy");

          if self.path.is_file() {
            return Ok(());
          };
        };

        whatever!("could not find module at {:?}", self.path);
      },
    }
  }

  fn source_bytes(&self) -> Result<Box<dyn Read>, Whatever> {
    match &self.kind {
      FileKind::EmbeddedStdSource => {
        let Some(file) = get_stl_source(&self.path) else {
          whatever!(
            "could not locate module at res://{}",
            self.path.to_string_lossy()
          );
        };

        Ok(Box::new(file))
      },
      FileKind::SourceFile => {
        let Ok(file) = File::open(&self.path) else {
          panic!("failed to open file");
        };

        let buf_reader = BufReader::new(file);
        Ok(Box::new(buf_reader))
      },
    }
  }

  pub(super) fn source_chars(&self) -> Result<impl Iterator<Item = (char, usize)>, Whatever> {
    let bytes = self.source_bytes()?;
    let mut utf8_reader = utf8_read::Reader::new(bytes);

    Ok(std::iter::from_fn(move || {
      let index = utf8_reader.borrow_pos().byte();

      let Char::Char(ch) = utf8_reader.next_char().expect("failed to decode utf-8") else {
        return None;
      };

      Some((ch, index))
    }))
  }
}
