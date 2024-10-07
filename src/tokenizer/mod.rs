use crate::compiler::{
  Compiler,
  TakenCompilerModule,
  CompilerResult,
  CompilerWorkflow,
};

#[allow(unused)]
#[derive(Debug)]
pub(crate) enum TokenKind {
  Whitespace,
  Identifier,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Span {
  start: usize,
  end: usize,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Token {
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
  Identifier { start: SpanStart, content: String, },
}

macro_rules! whitespace {
  () => { ' ' | '\t' | '\r' |  '\n' };
}

macro_rules! ident {
  () => { 'a'..='z' | 'A'..='Z' | '_' };
}

macro_rules! octal {
  () => { '0'..='7' };
}

macro_rules! decimal {
  () => { '0'..='9' }
}

macro_rules! hexademical {
  () => { decimal!() | 'a'..='f' | 'A'..='F' };
}

pub(super) struct Tokenizer {
  state: State,
  tokens: Vec<Token>,
}

#[derive(Debug, Clone, Copy)]
struct ReaderItem {
  position: usize,
  ch: char,
}

struct PeekReader<'a> {
  reader: &'a mut dyn Iterator<Item = CompilerResult<ReaderItem>>,
  peek_buffer: Option<ReaderItem>,
}

impl<'a> PeekReader<'a> {
  fn new(reader: &'a mut dyn Iterator<Item = CompilerResult<ReaderItem>>) -> Self {
    Self { reader, peek_buffer: None, }
  }

  fn seek(&mut self) {
    if self.peek_buffer.is_some() {
      self.peek_buffer = None;
    } else {
      self.next();
    };
  }

  fn peek(&mut self) -> CompilerResult<Option<ReaderItem>> {
    if let Some(buffered) = self.peek_buffer {
      return Ok(Some(buffered));
    };

    let Some(item) = self.next() else {
      return Ok(None);
    };

    let item = item?;
    self.peek_buffer = Some(item);

    Ok(Some(item))
  }
}

impl Iterator for PeekReader<'_> {
  type Item = CompilerResult<ReaderItem>;

  fn next(&mut self) -> Option<Self::Item> {
    let (message, result) = {
      if let Some(buffered) = self.peek_buffer {
        self.peek_buffer = None;

        ("buffered", Some(Ok(buffered)))
      } else {
        ("read", self.reader.next())
      }
    };

    trace!("PeekReader::next {message}   \t-> {result:?}");
    result
  }
}

impl Tokenizer {
  fn whitespace(&mut self, reader: &mut PeekReader) -> CompilerResult<()> {
    let Some(item) = reader.next() else {
      return Ok(());
    };

    let item = item?;

    let start = SpanStart(item.position);
    let mut end = item.position;

    while let Some(item) = reader.peek()? {
      let whitespace!() = item.ch else {
        break;
      };

      end = item.position;
      reader.seek();
    };

    self.tokens.push(Token {
      kind: TokenKind::Whitespace,
      span: start.into_span(end),
    });

    Ok(())
  }

  fn identifier(&mut self, reader: &mut PeekReader) -> CompilerResult<()> {
    let Some(item) = reader.next() else {
      return Err("expected an identifier".into());
    };
    let item = item?;
    let start = SpanStart(item.position);

    let ident!() = item.ch else {
      return Err("expected an ident".into());
    };

    let mut content = String::from(item.ch);
    let mut end = item.position;

    loop {
      let Some(item) = reader.peek()? else {
        break;
      };

      end = item.position;

      let (ident!() | decimal!()) = item.ch else {
        break;
      };

      reader.seek();
      content.push(item.ch);
    };

    self.tokens.push(Token {
      kind: TokenKind::Identifier,
      span: start.into_span(end),
    });

    Ok(())
  }

  fn base(&mut self, reader: &mut PeekReader) -> CompilerResult<()> {
    trace!("Tokenizer::base");

    let Some(item) = reader.peek()? else {
      return Ok(());
    };

    match item.ch {
      whitespace!() => self.whitespace(reader)?,
      ident!() => self.identifier(reader)?,
      _ => todo!("{:?}", item.ch),
    };

    Ok(())
  }
}

impl<W: CompilerWorkflow> crate::compiler::Tokenize<W> for Tokenizer {
  type Out = Vec<Token>;

  fn new() -> Self {
    Self {
      state: State::Base,
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
