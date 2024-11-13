mod peek_reader;
mod token;
#[macro_use]
mod patterns;

mod impls;

use std::marker::PhantomData;

use peek_reader::{PeekReader, ReaderItem};
pub(crate) use token::*;

use crate::compiler::CompilerStoreHandle;
use crate::compiler::{error::IOSnafu, Compiler, CompilerWorkflow, TakenCompilerModule, Tokenize};
use crate::Result;

#[derive(Debug)]
pub(crate) struct Tokenizer<W: CompilerWorkflow> {
  module: TakenCompilerModule<W>,
  handle: CompilerStoreHandle<W>,
  tokens: Vec<Token<W>>,
  marker: PhantomData<W>,
}

impl<W: CompilerWorkflow> Tokenizer<W> {
  fn push_tok(&mut self, kind: TokenKind, start: SpanStart<W>, end: usize) {
    assert!(
      end >= start.start,
      "invalid span: {} >= {} == false: {kind:?}",
      start.start,
      end
    );

    let token = Token {
      kind,
      span: start.into_span(end),
    };

    self.tokens.push(token);
  }
}

impl<W: CompilerWorkflow> Tokenize<W> for Tokenizer<W> {
  type Out = Vec<Token<W>>;

  fn new(module: TakenCompilerModule<W>, handle: CompilerStoreHandle<W>) -> Self {
    Self {
      module,
      handle,
      tokens: vec![],
      marker: Default::default(),
    }
  }

  fn tokenize(mut self, compiler: &mut Compiler<W>) -> Result<Self::Out> {
    let path = &compiler.store.get_module(&self.module.handle).path;
    let file = path.bytes()?;

    let buf_reader = std::io::BufReader::new(file);

    let mut reader = utf8_read::Reader::new(buf_reader);
    let mut reader = reader
      .into_iter()
      .enumerate()
      .map(|(position, ch)| match ch {
        Ok(ch) => Ok(ReaderItem { position, ch }),
        Err(err) => IOSnafu {
          err: err.to_string(),
        }
        .fail()?,
      });

    let mut reader = PeekReader::new(&mut reader, self.handle);

    while reader.peek()?.is_some() {
      self.base(&mut reader)?;
    }

    Ok(self.tokens)
  }
}
