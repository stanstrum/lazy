use std::fs::File;
use std::path::PathBuf;

use crate::*;

#[derive(Debug)]
pub(crate) enum SourceFileData {
  Unparsed,
  // TODO
}

#[derive(Debug)]
pub(crate) struct SourceFile {
  path: PathBuf,
  data: SourceFileData,
}

#[derive(Debug, Clone, Copy)]
pub(crate)struct Handle {
  id: usize,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Compiler {
  handle_counter: usize,
  files: Vec<SourceFile>,
  entry_point: Handle,
}

impl SourceFile {
  fn new(path: PathBuf) -> Self {
    Self {
      path,
      data: SourceFileData::Unparsed
    }
  }
}

#[allow(unused)]
impl Compiler {
  fn get_unique_handle_id(&mut self) -> usize {
    let id = self.handle_counter;
    self.handle_counter += 1;

    id
  }

  pub(crate) fn create_handle(&mut self, source_file: SourceFile) -> Handle {
    let id = self.get_unique_handle_id();
    self.files.insert(id, source_file);

    Handle { id }
  }

  pub(crate) fn new(entry_point: PathBuf) -> Self {
    let entry_file = SourceFile::new(entry_point);

    Self {
      // handle counter begins at one because the zeroth handle is always the
      // entry point
      handle_counter: 1,
      files: vec![entry_file],
      entry_point: Handle { id: 0 },
    }
  }

  pub(crate) fn get_handle(&self, handle: &Handle) -> Option<&SourceFile> {
    self.files.get(handle.id)
  }

  // pub(crate) fn get_handle_mut(&mut self, handle: &Handle) -> Option<&mut SourceFile> {
  //   self.files.get_mut(handle.id)
  // }

  pub(crate) fn compile(&mut self) -> Result<(), CompilationError> {
    self.compile_handle(&self.entry_point.to_owned())
  }

  pub(crate) fn compile_handle(&mut self, handle: &Handle) -> Result<(), CompilationError> {
    let Some(SourceFile { path, data: SourceFileData::Unparsed }) = self.get_handle(handle) else {
      unreachable!();
    };

    // TODO: for some reason, this doesn't error when opening a directory :/
    let input_file = match File::open(path) {
      Ok(file) => file,
      Err(error) => {
        return InputFileSnafu { error }.fail();
      }
    };

    let mut reader = utf8_read::Reader::new(input_file);

    let (source, tokens) = match tokenizer::tokenize(handle, &mut reader) {
      Ok(result) => result,
      Err(error) if matches!(error, tokenizer::TokenizationError::InvalidSource { .. }) => {
        let tokenizer::TokenizationError::InvalidSource { parsed, source, .. } = &error else {
          unreachable!();
        };

        let color_stream = tokenizer::create_color_stream(parsed);

        crate::pretty_print_error(&error, source, color_stream);

        return Err(error.into());
      },
      Err(error) => {
        return Err(error.into());
      },
    };

    // let source = tokenizer::stringify(&tokens);
    let color_stream = tokenizer::create_color_stream(&tokens);

    // debug::tokens(&tokens);

    #[allow(unused_variables)]
    let ast = {
      match asterizer::asterize(handle, tokens) {
        Ok(ast) => ast,
        Err(error) => {
          crate::pretty_print_error(&error, &source, color_stream);

          return AsterizationSnafu { error }.fail();
        },
      }
    };

    // debug::ast(&ast);

    Ok(())
  }
}
