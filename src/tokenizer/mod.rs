use crate::compiler::{
  Compiler,
  TakenCompilerModule,
  CompilerResult,
  CompilerWorkflow,
};

#[allow(unused)]
#[derive(Debug)]
enum TokenKind {
  Whitespace,
  Identifier,
}

#[allow(unused)]
#[derive(Debug)]
struct Span {
  start: usize,
  end: usize,
}

#[allow(unused)]
#[derive(Debug)]
struct Token {
  kind: TokenKind,
  span: Span,
}

#[derive(Debug)]
struct SpanStart(usize);

impl SpanStart {
  fn into_span(&self, end: usize) -> Span {
    Span {
      start: self.0,
      end,
    }
  }
}

#[derive(Debug)]
enum State {
  Base,
  Whitespace { start: SpanStart, },
}

macro_rules! whitespace {
  () => {
    ' ' | '\t' | '\r' |  '\n'
  };
}

pub(super) struct Tokenizer;

impl<W: CompilerWorkflow> crate::compiler::Tokenize<W> for Tokenizer {
  type Out = ();

  fn tokenize(compiler: &mut Compiler<W>, module: TakenCompilerModule<W>) -> CompilerResult<Self::Out> {
    let path = compiler.store.get_module(&module.handle).path.as_path();

    let buf_reader = std::io::BufReader::new(
      std::fs::File::open(path)
        .map_err(|err| err.to_string())?
    );

    let mut reader = utf8_read::Reader::new(buf_reader);
    let mut state = State::Base;
    let mut tokens = vec![];

    for (position, ch) in reader.into_iter().enumerate() {
      let ch = match ch {
        Ok(x) => x,
        Err(err) => return Err(err.to_string()),
      };

      match (&state, ch) {
        (State::Base, whitespace!()) => {
          state = State::Whitespace {
            start: SpanStart(position),
          };
        },
        (State::Whitespace { .. }, whitespace!()) => (),
        (State::Whitespace { start }, ..) => {
          let token = Token {
            kind: TokenKind::Whitespace,
            span: start.into_span(position),
          };

          debug!("New token: {token:?}");

          let value = token;

          tokens.push(value)
        },
        _ => todo!("({state:?}, {ch:?})"),
      };
    };

    todo!()
  }
}
