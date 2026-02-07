use std::cmp::Ordering;

use crate::lang::module::ModulePath;
use crate::lang::reference::Store;
use crate::tokenize::token::{Position, Token, TokenSpan};

use super::*;

#[macro_export]
macro_rules! colorize {
  ($color:expr) => {
    concat!("\x1b[", stringify!($color), "m")
  };
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

pub fn print_message(lazy: &Lazy, message: PrintableMessage) {
  // TODO: Add settings (incl. log level) to `Lazy`

  // If this message isn't being force-printed, check if we
  // should actually print it
  if !message.force && message.level < lazy.settings.log_level {
    // Don't print.
    return;
  };

  // Output buffer for building the message
  let mut out = vec![];

  // "info: this is a message"
  writeln!(&mut out, "{bold}{level}{clear} {desc}",
    bold = colorize!(1),
    level = message.level,
    clear = colorize!(0),
    desc = message.description,
  ).unwrap();

  // if
  let MessageContents::WithinSource { range, sections } = message.contents;
  // {
  // };
  print_sections(&mut out, lazy, range, sections);

  let out = std::str::from_utf8(&out).expect("output parsed as utf-8");
  print!("{out}");
}

struct Colorizer<'a> {
  tokens: &'a [TokenSpan],
}

impl<'a> Colorizer<'a> {
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
    }
  }

  fn find_token_by(&mut self, line: usize, column: usize) -> Option<&TokenSpan> {
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

fn print_sections(out: &mut Vec<u8>, lazy: &Lazy, range: Span, mut sections: Vec<MessageSection>) {
  // Open and create a reader for this module's source file
  let ModulePath { path, tokens } = lazy.get_path(range.module);
  let file = File::open(path).unwrap();
  let mut reader = BufReader::new(file);

  // Get reference to the tokens saved by the asterizer
  let tokens = lazy.rget(*tokens);
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

    writeln!(out, "  --> {path}:{line}:{col}",
      path = {
        let mut path = lazy.get_path(section.span.module).path.as_path();

        if
          let Some(parent) = lazy.settings.input_path.parent() &&
          let Ok(stripped) = path.strip_prefix(parent)
        {
          path = stripped;
        };

        path.to_string_lossy()
      },
      line = section.span.start.line,
      col = section.span.start.column,
    ).unwrap();

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
