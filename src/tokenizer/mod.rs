mod token;
mod peek_reader;
#[macro_use] mod patterns;

mod impls;

pub(self) use peek_reader::PeekReader;
use peek_reader::ReaderItem;
pub(crate) use token::*;

use crate::compiler::{
  Compiler,
  TakenCompilerModule,
  CompilerResult,
  CompilerWorkflow,
};

pub(super) struct Tokenizer {
  tokens: Vec<Token>,
}

impl Tokenizer {
  fn push_tok(&mut self, kind: TokenKind, start: SpanStart, end: usize) {
    let token = Token {
      kind,
      span: start.into_span(end),
    };

    debug!("Tokenizer::push_tok {token:?}");

    self.tokens.push(token);
  }
}

impl<W: CompilerWorkflow> crate::compiler::Tokenize<W> for Tokenizer {
  type Out = Vec<Token>;

  fn new() -> Self {
    Self {
      tokens: vec![],
    }
  }

  fn tokenize(mut self, compiler: &mut Compiler<W>, module: TakenCompilerModule<W>) -> CompilerResult<Self::Out> {
    let path = compiler.store.get_module(&module.handle).path.as_path();

    let buf_reader = std::io::BufReader::new(
      std::fs::File::open(path)
        .map_err(|err| err.to_string())?
    );

    let mut reader = utf8_read::Reader::new(buf_reader);
    let mut reader = reader
      .into_iter()
      .enumerate()
      .map(|(position, ch)| match ch {
        Ok(ch) => Ok(ReaderItem { position, ch, }),
        Err(err) => Err(err.to_string()),
      });

    let mut reader = PeekReader::new(&mut reader);

    while reader.peek()?.is_some() {
      self.base(&mut reader)?;
    };

    Ok(self.tokens)
  }
}
