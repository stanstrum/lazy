mod seek;

use std::io::{BufRead, BufReader, Write};
use std::fs::File;
use std::collections::HashMap;
use std::cmp::Ordering;

use lazy_macros::{colorize, line_dbg};
use lang::token::Token;
use lang::span::{Position, Span};
use lang::reference::Store;
use lang::{Compiler, CompilerPoolStore};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(u8)]
#[allow(unused)]
pub enum Level {
  Debug,
  Stub,
  Info,
  Warn,
  Error,
}

#[derive(Debug)]
pub struct PrintableMessage<C: Compiler> {
  pub level: Level,
  pub force: bool,
  pub description: String,
  pub contents: MessageContents<C>,
}

#[derive(Debug)]
pub struct WithinSource<C: Compiler> {
  pub range: Span<C>,
  pub sections: Vec<MessageSection<C>>,
}

#[derive(Debug)]
pub enum MessageContents<C: Compiler> {
  WithinSource(Vec<WithinSource<C>>),
  File(C::ModuleReference),
  None,
}

#[derive(Debug)]
pub struct MessageSection<C: Compiler> {
  pub text: String,
  pub span: Span<C>,
}

impl std::fmt::Display for Level {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(match self {
      Level::Debug => concat!(              colorize!(36), "debug", colorize!(0)),
      Level::Stub =>  concat!(colorize!(7), colorize!(3),  "stub" , colorize!(0)),
      Level::Info =>  concat!(colorize!(7), colorize!(92), "info" , colorize!(0)),
      Level::Warn =>  concat!(colorize!(7), colorize!(93), "warn" , colorize!(0)),
      Level::Error => concat!(colorize!(7), colorize!(91), "error", colorize!(0)),
    })
  }
}

impl<C: Compiler> WithinSource<C> {
  pub fn new(sources: Vec<MessageSection<C>>) -> Vec<Self> {
    let mut map = HashMap::new();

    for section in sources {
      let list = map.entry(section.span.module).or_insert_with(Vec::new);
      list.push(section);
    };

    let mut results = Vec::with_capacity(map.len());
    for mut sources in map.into_values() {
      sources.sort_by_key(|section| section.span.start.position);

      assert!(!sources.is_empty());
      let start = sources.first().unwrap().span;
      let end = sources.last().unwrap().span;

      let range = Span::from_pair(start, end);

      results.push(WithinSource {
        range,
        sections: sources,
      });
    };

    results
  }
}

struct LineYielder {
  reader: BufReader<File>,
  line: usize,
  indentation: usize,
  finished: bool,
}

impl LineYielder {
  fn new(reader: BufReader<File>, line: usize) -> Self {
    Self {
      reader,
      line,
      indentation: 0,
      finished: false,
    }
  }

  fn rewind(&mut self, pos: Position) {
    seek::find_starting_newline(&mut self.reader, pos.position);
    self.line = pos.line;
  }
}

impl Iterator for LineYielder {
  type Item = String;

  fn next(&mut self) -> Option<Self::Item> {
    if self.finished {
      return None;
    };

    let mut buffer = String::new();
    let remaining = self.reader.read_line(&mut buffer)
      .expect("read line");

    if remaining == 0 {
      self.finished = true;
    };

    buffer.truncate(buffer.trim_end_matches(['\r', '\n']).len());

    self.indentation = buffer.chars()
      .position(|ch| !matches!(ch, ' ' | '\t'))
      .unwrap_or(0);

    self.line += 1;
    Some(buffer)
  }
}

struct Colorizer<'a, C: Compiler> {
  tokens: &'a [lang::token::TokenSpan<C>],
}

impl<'a, C: Compiler> Colorizer<'a, C> {
  fn colorize(token: &Token) -> &'static str {
    match token {
      Token::Identifier(_) => colorize!(93),
      Token::Keyword(_) => colorize!(91),
      Token::Operator(_) => colorize!(97),
      Token::Grouping(_) => colorize!(94),
      Token::Whitespace => colorize!(0),
      Token::Indent(_) => colorize!(36),
      Token::Comment(_) => colorize!(37),
      Token::Numeric(_) => colorize!(34),
      Token::String(..) => colorize!(31),
    }
  }

  fn find_token_by(&mut self, line: usize, column: usize) -> Option<&lang::token::TokenSpan<C>> {
    let offset = self.tokens.iter()
      .position(|(_, span)| span.start.line == line)?;

    self.tokens = &self.tokens[offset..];

    let tok_iter = self.tokens.iter()
      .take_while(|(_, span)| span.start.line == line);
    for token_span @ (_, span) in tok_iter {
      if column < span.start.column {
        continue;
      };

      if column >= span.end.column {
        continue;
      };

      return Some(token_span);
    };

    None
  }

  fn write(&mut self, mut out: impl Write, line: usize, text: &str) {
    let mut column = 1;
    while column <= text.len() {
      if let Some((token, span)) = self.find_token_by(line, column) {
        let color = Self::colorize(token);
        write!(out, "{color}").unwrap();

        let length = span.end.column - column;

        write!(out, "{}", &text[(column - 1)..][..length]).unwrap();
        write!(out, colorize!(0)).unwrap();
        column += length;

        continue;
      };

      write!(out, "{}", text.chars().nth(column - 1).unwrap()).unwrap();
      column += 1;
    };
  }
}

pub fn print_message<C: Compiler>(store: &C::Store<'_>, message: PrintableMessage<C>) {
  // TODO: Add settings (incl. log level) to `Lazy`

  // SPONGE: move this code into the macro
  // // If this message isn't being force-printed, check if we
  // // should actually print it
  // if !message.force && message.level < store.settings.log_level {
  //   // Don't print.
  //   return;
  // };

  // Output buffer for building the message
  let mut out = vec![];

  // "info: this is a message"
  writeln!(&mut out, "{bold}{level}{clear} {desc}",
    bold = colorize!(1),
    level = message.level,
    clear = colorize!(0),
    desc = message.description,
  ).unwrap();

  match message.contents {
    MessageContents::WithinSource(within_source) => {
      for WithinSource { range, sections } in within_source {
        print_sections(&mut out, store, range, sections);
      };
    },
    MessageContents::File(module) => {
      print_partial_section_header::<C>(&mut out, store, module);
    },
    MessageContents::None => {},
  };

  let out = std::str::from_utf8(&out).expect("output parsed as utf-8");
  print!("{out}");
}

fn print_section_header<C: Compiler>(out: &mut Vec<u8>, store: &C::Store<'_>, module: C::ModuleReference, position: Option<Position>) {
  let mut path = store.get_path(module).path.as_path();

  // SPONGE: reimplement this elsewhere
  // if
  //   let Some(parent) = store.settings.input_path.parent() &&
  //   let Ok(stripped) = path.strip_prefix(parent)
  // {
  //   path = stripped;
  // };

  write!(out, "  --> {}", path.to_string_lossy()).unwrap();

  if let Some(position) = position {
    write!(out, ":{line}:{col}",
      line = position.line,
      col = position.column,
    ).unwrap();
  };

  writeln!(out).unwrap();
}

fn print_partial_section_header<C: Compiler>(out: &mut Vec<u8>, store: &C::Store<'_>, module: C::ModuleReference) {
  print_section_header::<C>(out, store, module, None)
}

fn print_full_section_header<C: Compiler>(out: &mut Vec<u8>, store: &C::Store<'_>, span: lang::span::Span<C>) {
  print_section_header::<C>(out, store, span.module, Some(span.start))
}

fn print_sections<C: Compiler>(out: &mut Vec<u8>, store: &C::Store<'_>, range: lang::span::Span<C>, mut sections: Vec<MessageSection<C>>) {
  // Open and create a reader for this module's source file
  let lang::module::ModulePath::<C> { path, tokens, .. } = store.get_path(range.module);
  let file = File::open(path).unwrap();
  let mut reader = BufReader::new(file);

  // Get reference to the tokens saved by the asterizer
  let tokens = store.rget(*tokens).as_slice();
  let mut colorizer = Colorizer { tokens };

  // We'll track where we are in the file once we start
  // moving around for coloring the code, printing the
  // line numbers reliably, and finally underlining relevant
  // parts of the code.

  // // We'll be making sure this is the case.
  // let mut column = 1;

  // The provided range might not land on the character
  // after a newline, i.e., in the middle of a line.  Find
  // the preceding newline, if there is one.
  let _end_position = seek::find_ending_newline(
    &mut reader,
    range.end.position,
  );

  let _start_position = seek::find_starting_newline(
    &mut reader,
    range.start.position
  );

  // Sort our sections so we can print them in progressive order
  sections.sort_by_key(|section| section.span.start.position);

  // String length of the greatest line number we find
  let number_length = (range.end.line.ilog10() + 1) as usize;
  let number_padding = " ".repeat(number_length);

  // Keep track of lines and yield one at a time
  let mut yielder = LineYielder::new(reader, range.start.line);

  // Print each section
  for section in sections.iter() {

    if yielder.line > section.span.start.line {
      yielder.rewind(section.span.start);
    };

    print_full_section_header(out, store, section.span);

    // Skip at least until the line before
    while yielder.line + 1 < section.span.start.line {
      // skip line. TODO: this allocates a buffer and should be replaced w/ a
      // method that just drops these values
      yielder.next();
    };

    // Print the padding line(s) until we arrive
    while let line = yielder.line && line < section.span.start.line &&
      let Some(line_text) = yielder.next()
    {
      write!(out, " {line:>number_length$} {bold}|{clear} ",
        bold = colorize!(1),
        clear = colorize!(0),
      ).unwrap();
      colorizer.write(&mut *out, yielder.line, &line_text);
      writeln!(out).unwrap();
    };

    // Sanity check, that we are where we think we are
    assert!(yielder.line == section.span.start.line);

    // Print the lines in question, plus squiggles
    let lines_to_print = section.span.end.line - section.span.start.line;
    for _ in 0..=lines_to_print {
      let line = yielder.line;
      let line_text = yielder.next().expect("yield line");
      write!(out, " {bold}{line:>number_length$} |{clear} ",
        bold = colorize!(1),
        clear = colorize!(0),
      ).unwrap();
      colorizer.write(&mut *out, line, &line_text);
      writeln!(out).unwrap();

      let line_length = line_text.len();
      let squiggle_start = match line.cmp(&section.span.start.line) {
        Ordering::Less => panic!("out of bounds"),
        Ordering::Equal => section.span.start.column,
        Ordering::Greater => yielder.indentation + 1,
      };

      let squiggle_end = match line.cmp(&section.span.end.line) {
        Ordering::Less => {
          if line_length != 0 {
            line_length + 1
          } else {
            0
          }
        },
        Ordering::Equal => section.span.end.column,
        Ordering::Greater => panic!("out of bounds"),
      };

      let mut squiggle_text = (1..squiggle_end).map(|column| {
        if (squiggle_start..squiggle_end).contains(&column) {
          '^'
        } else {
          ' '
        }
      }).collect::<String>();

      if squiggle_start == squiggle_end {
        squiggle_text.push('^');
      };

      writeln!(out, " {number_padding} {bold}|{clear} {squiggle_text} {msg}",
        bold = colorize!(1),
        clear = colorize!(0),
        msg = if line == section.span.end.line { &section.text } else { "" },
      ).unwrap();
    };
  };
}

impl<C: Compiler> From<lang::error::TokenError<C>> for PrintableMessage<C> {
  fn from(value: lang::error::TokenError<C>) -> Self {
    match value {
      lang::error::TokenError::IO { name, module } => Self {
        level: Level::Error,
        force: true,
        description: format!(line_dbg!("IO error for {}"), name),
        contents: MessageContents::File(module),
      },
      lang::error::TokenError::InvalidNumeric { span } => Self {
        level: Level::Error,
        force: true,
        description: line_dbg!("Invalid numeric").into(),
        contents: MessageContents::WithinSource(vec![WithinSource {
          range: span,
          sections: vec![MessageSection {
            text: "here".into(),
            span,
          }],
        }]),
      },
    }
  }
}

impl<C: Compiler> From<lang::error::AsterError<C>> for PrintableMessage<C> {
  fn from(value: lang::error::AsterError<C>) -> Self {
    match value {
      lang::error::AsterError::Token(error) => error.into(),
      lang::error::AsterError::Expected { what, at } => Self {
        level: Level::Error,
        force: true,
        description: format!(line_dbg!("expected {}"), what),
        contents: MessageContents::WithinSource(vec![WithinSource {
          range: at,
          sections: vec![MessageSection {
            text: "here".into(),
            span: at,
          }],
        }]),
      },
      lang::error::AsterError::Invalid { what, at } => Self {
        level: Level::Error,
        force: true,
        description: format!(line_dbg!("invalid {}"), what),
        contents: MessageContents::WithinSource(vec![WithinSource {
          range: at,
          sections: vec![MessageSection {
            text: "here".into(),
            span: at,
          }],
        }]),
      },
      lang::error::AsterError::Lazy(lazy) => (*lazy).into(),
    }
  }
}

impl<C: Compiler> From<Box<lang::error::ResolveError<C>>> for PrintableMessage<C> {
  fn from(value: Box<lang::error::ResolveError<C>>) -> Self {
    #[cfg(debug_assertions)]
    println!("{}", value.call_stack);

    match value.base {
      lang::error::ResolveErrorBase::UnknownTypeName { module_name, span } => Self {
        level: Level::Error,
        force: true,
        description: format!(line_dbg!("unknown type name in {}"), module_name),
        contents: MessageContents::WithinSource(vec![WithinSource {
          range: span,
          sections: vec![MessageSection {
            text: "here".into(),
            span,
          }],
        }]),
      },
      lang::error::ResolveErrorBase::MissingEntryPoint { module_name, file } => Self {
        level: Level::Error,
        force: true,
        description: format!(line_dbg!("{:?} is missing an entry point!"), module_name),
        contents: MessageContents::File(file),
      },
      lang::error::ResolveErrorBase::TypeMismatch {
        whence,
        a_print, a_span,
        b_print, b_span,
      } => {
        let a_section = MessageSection {
          text: "here".into(),
          span: a_span,
        };

        let b_section = MessageSection {
          text: "here".into(),
          span: b_span,
        };

        let mut sections = vec![a_section];

        if a_span != b_span {
          sections.push(b_section);
        };

        let within_sources = WithinSource::new(sections);

        Self {
          level: Level::Error,
          force: true,
          description: format!(line_dbg!("{}{} is not coercible with {}"), whence, a_print, b_print),
          contents: MessageContents::WithinSource(within_sources),
        }
      },
      lang::error::ResolveErrorBase::UnresolvedInVerify { what, span } => Self {
        level: Level::Error,
        force: true,
        description: format!(line_dbg!("verify error: {} is not resolved"), what),
        contents: MessageContents::WithinSource(WithinSource::new(
          vec![MessageSection {
            text: "here".into(),
            span,
          }],
        )),
      },
      lang::error::ResolveErrorBase::BadQualify { span } => Self {
        level: Level::Error,
        force: true,
        description: line_dbg!("invalid part in qualifier").into(),
        contents: MessageContents::WithinSource(WithinSource::new(
          vec![MessageSection {
            text: "here".into(),
            span,
          }],
        )),
      },
      lang::error::ResolveErrorBase::NotImplemented { what, span } => Self {
        level: Level::Error,
        force: true,
        description: format!("not implemented: {what}"),
        contents: MessageContents::WithinSource(WithinSource::new(
          vec![MessageSection {
            text: "here".into(),
            span,
          }],
        )),
      },
      lang::error::ResolveErrorBase::Lazy(lazy_error) => (*lazy_error).into(),
    }
  }
}

impl<C: Compiler> From<lang::error::LazyError<C>> for PrintableMessage<C> {
  fn from(value: lang::error::LazyError<C>) -> Self {
    match value {
      lang::error::LazyError::NotExist(path_buf) => Self {
        level: Level::Error,
        force: true,
        description: format!(line_dbg!("not a file {:?}"), &path_buf),
        contents: MessageContents::None,
      },
      lang::error::LazyError::Aster(aster) => aster.into(),
    }
  }
}
