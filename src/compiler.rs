use std::fs::File;
use std::collections::HashMap;
use std::path::PathBuf;

use asterizer::ast::GlobalNamespace;
use tokenizer::Token;

use colors::Color;

use typechecker::{
  TypeChecker,
  Domain,
  Program,
};

use crate::*;

#[derive(Debug)]
pub(crate) struct DebugInfo {
  source: String,
  color_stream: Vec<(usize, Color)>
}

#[derive(Debug)]
pub(crate) enum SourceFileData {
  Borrowed,
  Unparsed,
  Tokenized(Vec<Token>),
  Asterized(GlobalNamespace),
  TypeChecked(Domain),
  // TODO
}

impl Default for SourceFileData {
  fn default() -> Self {
    Self::Borrowed
  }
}

#[derive(Debug)]
pub(crate) struct SourceFile {
  pub(crate) path: PathBuf,
  pub(crate) data: SourceFileData,
  pub(crate) debug_info: Option<DebugInfo>,
}

impl SourceFile {
  fn open(&self) -> Result<File, CompilationError> {
    match File::open(&self.path) {
      Ok(file) => Ok(file),
      Err(error) => InputFileSnafu { error }.fail()
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Handle {
  pub(crate) id: usize,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Compiler {
  handle_counter: usize,
  // TODO: using a singly-linked list may be better here given that we never need to
  //       read backwards and we remove/reinsert elements at each step of compilation
  //       so we don't need to worry about null elements that are being replaced --
  //       also removes the need for constant copying/cloning path bufs
  files: Vec<SourceFile>,
  pub(crate) entry_point: Handle,
}

impl SourceFile {
  pub(crate) fn new(path: PathBuf) -> Self {
    Self {
      path,
      data: SourceFileData::Unparsed,
      debug_info: None
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

  // TODO: deduplicate the same file path
  pub(crate) fn create_handle(&mut self, mut source_file: SourceFile) -> Handle {
    let metadata = std::fs::metadata(&source_file.path);

    if metadata.is_ok_and(|metadata| metadata.is_dir()) {
      source_file.path.push("index.zy");
    };

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

  pub(crate) fn compile(mut self) -> Result<(), CompilationError> {
    self.compile_handle(self.entry_point)?;

    let mut checker = TypeChecker::new(self.entry_point);

    for id in 0..self.files.len() {
      let handle = Handle { id };

      let borrowed_file = self.take_handle(id);
      let result = checker.preprocess(borrowed_file, &handle)?;

      self.replace_handle(id, result);
    };

    let mut program_map = HashMap::new();
    for id in 0..self.files.len() {
      let handle = Handle { id };
      let SourceFile {
        path,
        data: SourceFileData::TypeChecked(domain),
        debug_info,
      } = self.take_handle(handle.id) else {
        panic!("cannot typecheck: not all files preprocessed");
      };

      // TODO: preserve debug info here
      program_map.insert(handle, domain);
    };

    let program = Program::from(program_map);

    checker.check(program).map_err(Into::into)
  }

  fn take_handle(&mut self, id: usize) -> SourceFile {
    let file_ref = &mut self.files[id];

    let data = std::mem::take(&mut file_ref.data);
    let debug_info = std::mem::take(&mut file_ref.debug_info);

    SourceFile {
      path: file_ref.path.to_owned(),
      debug_info,
      data,
    }
  }

  fn replace_handle(&mut self, id: usize, file: SourceFile) {
    self.files[id] = file;
  }

  fn borrow_handle(&mut self, handle: &Handle, processor: fn(&mut Compiler, SourceFile, &Handle) -> Result<SourceFile, CompilationError>) -> Result<(), CompilationError> {
    let borrowed_file = self.take_handle(handle.id);
    let result = processor(self, borrowed_file, handle)?;

    self.replace_handle(handle.id, result);

    Ok(())
  }

  // TODO: move this into tokenizer
  fn tokenize_file(&mut self, file: SourceFile, handle: &Handle) -> Result<SourceFile, CompilationError> {
    let mut reader = utf8_read::Reader::new(file.open()?);

    let (source, tokens) = match tokenizer::tokenize(handle, &mut reader) {
      Ok(result) => result,
      Err(error) if matches!(error, tokenizer::TokenizationError::InvalidSource { .. }) => {
        let tokenizer::TokenizationError::InvalidSource { parsed, source, .. } = &error else {
          unreachable!();
        };

        let color_stream = tokenizer::create_color_stream(parsed);

        crate::pretty_print_error(&error, source, color_stream, &file.path);

        return Err(error.into());
      },
      Err(error) => {
        return Err(error.into());
      },
    };

    // let source = tokenizer::stringify(&tokens);
    let color_stream = tokenizer::create_color_stream(&tokens);

    let debug_info = Some(DebugInfo {
      source,
      color_stream,
    });

    Ok(SourceFile {
      path: file.path,
      data: SourceFileData::Tokenized(tokens),
      debug_info
    })
  }

  // TODO: rework how this works -- we shouldn't need an anonymous callback to turn this into
  //       a proper processor.  definitely make the compiler a singleton so we can stop playing
  //       a cat-and-mouse game with the borrow checker.  the compiler is always.
  // TODO: move this into asterizer
  fn asterize_file(&mut self, file: SourceFile, handle: &Handle) -> Result<SourceFile, CompilationError> {
    let SourceFile {
      path,
      debug_info,
      data: SourceFileData::Tokenized(tokens)
    } = file else {
      panic!("tried to asterize a non-tokenized file");
    };

    let ast = match asterizer::asterize(self, &path, handle, tokens) {
      Ok(ast) => ast,
      Err(error) => {
        let Some(DebugInfo { source, color_stream }) = debug_info else {
          panic!("no debug info");
        };

        crate::pretty_print_error(&error, &source, color_stream, &path);

        return AsterizationSnafu { error }.fail();
      },
    };

    Ok(SourceFile {
      path,
      debug_info,
      data: SourceFileData::Asterized(ast),
    })
  }

  // fn typecheck_file(&mut self, file: SourceFile, handle: &Handle) -> Result<SourceFile, CompilationError> {
  //   let SourceFile {
  //     path,
  //     debug_info,
  //     data: SourceFileData::Asterized(ast)
  //   } = file else {
  //     panic!("tried to asterize a non-tokenized file");
  //   };

  //   let program = match typechecker::typecheck(self, &path, handle, ast) {
  //     Ok(program) => program,
  //     Err(error) => {
  //       let Some(DebugInfo { source, color_stream }) = debug_info else {
  //         panic!("no debug info");
  //       };

  //       crate::pretty_print_error(&error, &source, color_stream, &path);

  //       return TypeCheckSnafu { error }.fail();
  //     },
  //   };

  //   Ok(SourceFile {
  //     path,
  //     debug_info,
  //     data: SourceFileData::Typechecked(program),
  //   })
  // }

  pub(crate) fn compile_handle(&mut self, handle: Handle) -> Result<(), CompilationError> {
    self.borrow_handle(&handle, Self::tokenize_file)?;
    self.borrow_handle(&handle, Self::asterize_file)?;

    for id in 0..self.files.len() {
      if matches!(&self.files[id].data, SourceFileData::Unparsed) {
        self.compile_handle(Handle { id })?;
      };

      // dbg!(id, &self.files[id].data);
    };

    Ok(())
  }
}
