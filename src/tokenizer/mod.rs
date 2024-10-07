mod consts;

use consts::{
  Grouping,
  Keyword,
  Operator,
  Punctuation,
};

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
  Identifier(String),
  Operator(Operator),
  Keyword(Keyword),
  Comment(String),
  Punctuation(Punctuation),
  Grouping(Grouping),
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

#[derive(Debug, Clone, Copy)]
struct SpanStart(usize);

impl SpanStart {
  fn into_span(&self, end: usize) -> Span {
    Span {
      start: self.0,
      end,
    }
  }
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

macro_rules! operator {
  () => { '~' | '!' | '%' | '^' | '&' | '-' | '+' | '=' | '|' | '<' | '>' | '/' | '?' | ':' | ';' | ',' | '.' };
}

pub(super) struct Tokenizer {
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
  position: usize,
}

impl<'a> PeekReader<'a> {
  fn new(reader: &'a mut dyn Iterator<Item = CompilerResult<ReaderItem>>) -> Self {
    Self {
      reader,
      peek_buffer: None,
      position: 0,
    }
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

  fn span_start(&self) -> SpanStart {
    SpanStart(self.position)
  }

  fn span_finish(&self, start: SpanStart) -> Span {
    start.into_span(self.position)
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
        let next = self.reader.next();

        if next.as_ref().is_some_and(|next| next.is_ok()) {
          self.position += 1;
        };

        ("read", next)
      }
    };

    trace!("PeekReader::next {message}   \t-> {result:?}");
    result
  }
}

impl Tokenizer {
  fn whitespace(&mut self, reader: &mut PeekReader) -> CompilerResult<()> {
    trace!("Tokenizer::whitespace");

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

    self.push_tok(TokenKind::Whitespace, start, end);

    Ok(())
  }

  fn identifier(&mut self, reader: &mut PeekReader) -> CompilerResult<()> {
    trace!("Tokenizer::identifier");

    let Some(item) = reader.next() else {
      return Err("expected an identifier".into());
    };
    let item = item?;

    let ident!() = item.ch else {
      return Err(format!("expected an identifier: {:?}", item.ch))
    };

    let start = SpanStart(item.position);
    let mut name = String::from(item.ch);

    loop {
      let Some(peek) = reader.peek()? else {
        break;
      };

      let (ident!() | decimal!()) = peek.ch else {
        break;
      };

      name.push(peek.ch);
      reader.seek();
    };

    let kind = if let Some(keyword) = Keyword::from_str(&name) {
      TokenKind::Keyword(keyword)
    } else {
      TokenKind::Identifier(name)
    };

    self.push_tok(kind, start, reader.position);

    Ok(())
  }

  fn line_comment(&mut self, reader: &mut PeekReader) -> CompilerResult<()> {
    trace!("Tokenizer::line_comment");
    let mut message = String::new();
    let start = reader.span_start();

    for item in &mut *reader {
      let item = item?;

      if let '\n' = item.ch {
        break;
      };

      message.push(item.ch);
    };

    self.push_tok(TokenKind::Comment(message.trim().into()), start, reader.position);

    Ok(())
  }

  fn push_tok(&mut self, kind: TokenKind, start: SpanStart, end: usize) {
    let token = Token {
      kind,
      span: start.into_span(end),
    };

    debug!("Tokenizer::push_tok {token:?}");

    self.tokens.push(token);
  }

  fn operator(&mut self, reader: &mut PeekReader) -> CompilerResult<()> {
    trace!("Tokenizer::operator");
    let Some(item) = reader.peek()? else {
      return Err("expected an operator".into());
    };

    let start = SpanStart(item.position);
    let mut end = start.0;

    let mut content = String::new();

    loop {
      let Some(item) = reader.next() else {
        break;
      };
      let item = item?;

      end = item.position;
      content.push(item.ch);

      let Some(peek) = reader.peek()? else {
        break;
      };

      match (content.as_str(), peek.ch) {
        | ("%", '=')
        | ("^", '^' | '=')
        | ("^^", '=')
        | ("&", '&' | '=')
        | ("&&", '=')
        | ("*", '*' | '=')
        | ("**", '=')
        | ("-", '=' | '-' | '>')
        | ("+", '=' | '+')
        | ("=", '=')
        | ("|", '|' | '=')
        | ("||", '|' | '=')
        | ("<", '<' | '=')
        | ("<<", '<' | '=')
        | ("<<<", '=')
        | (">", '>' | '=')
        | (">>", '>' | '=')
        | (">>>", '=')
        | ("/", '/' | '*' | '=')
        | (":", ':')
        | (".", '.')
        | ("..", '.')
        => {},
        ("//", _) => {
          self.line_comment(reader)?;
          return Ok(());
        },
        ("/*", _) => todo!("multiline comment"),
        _ => break,
      };
    };

    let kind = if let Some(op) = Operator::from_str(&content) {
      TokenKind::Operator(op)
    } else if let Some(punct) = Punctuation::from_str(&content) {
      TokenKind::Punctuation(punct)
    } else {
      return Err(format!("unrecognized operator: {content:?}"));
    };

    self.push_tok(kind, start, end);

    Ok(())
  }

  fn base(&mut self, reader: &mut PeekReader) -> CompilerResult<()> {
    trace!("Tokenizer::base");
    let start = reader.span_start();

    let Some(item) = reader.peek()? else {
      return Ok(());
    };

    if let Some(grouping) = Grouping::from_str(&String::from(item.ch)) {
      reader.seek();
      self.push_tok(TokenKind::Grouping(grouping), start, reader.position);
      return Ok(());
    };

    match item.ch {
      whitespace!() => self.whitespace(reader)?,
      ident!() => self.identifier(reader)?,
      operator!() => self.operator(reader)?,
      _ => todo!("{:?}", item.ch),
    };

    Ok(())
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
