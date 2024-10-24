mod token;
mod peek_reader;
#[macro_use] mod patterns;

mod impls;

use std::marker::PhantomData;

use peek_reader::{
  PeekReader,
  ReaderItem,
};
pub(crate) use token::*;

use crate::Result;
use crate::compiler::{
  Compiler,
  CompilerWorkflow,
  Tokenize,
  TakenCompilerModule,
  error::IOSnafu,
};

pub(super) struct Tokenizer<W: CompilerWorkflow> {
  module: TakenCompilerModule<W>,
  tokens: Vec<Token<W>>,
  marker: PhantomData<W>,
}

impl<W: CompilerWorkflow> Tokenizer<W> {
  fn push_tok(&mut self, kind: TokenKind, start: SpanStart<W>, end: usize) {
    debug!("Tokenizer::push_tok: {kind:?}");

    let token = Token {
      kind,
      span: start.into_span(end),
    };

    self.tokens.push(token);
  }
}

impl<W: CompilerWorkflow> Tokenize<W> for Tokenizer<W> {
  type Out = Vec<Token<W>>;

  fn new(module: TakenCompilerModule<W>) -> Self {
    Self {
      module,
      tokens: vec![],
      marker: Default::default(),
    }
  }

  fn tokenize(mut self, compiler: &mut Compiler<W>) -> Result<Self::Out> {
    let path = compiler.store.get_module(&self.module.handle).path.as_path();
    let file = match std::fs::File::open(path) {
      Ok(x) => x,
      Err(err) => return IOSnafu { err: err.to_string() }.fail()?,
    };

    let buf_reader = std::io::BufReader::new(file);

    let mut reader = utf8_read::Reader::new(buf_reader);
    let mut reader = reader
      .into_iter()
      .enumerate()
      .map(|(position, ch)| match ch {
        Ok(ch) => Ok(ReaderItem { position, ch, }),
        Err(err) => IOSnafu { err: err.to_string() }.fail()?,
      });

    let mut reader = PeekReader::new(&mut reader);

    while reader.peek()?.is_some() {
      self.base(&mut reader)?;
    };

    Ok(self.tokens)
  }
}
