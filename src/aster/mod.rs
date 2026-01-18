mod make;
pub mod bufreader;
pub mod rereader;
mod pprint;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use bufreader::BufferedUtf8MetadataReader;
use crate::aster::rereader::Rereader;
use crate::string_pool::StringPool;

use crate::tokenize::{self, Tokenizer};
use crate::tokenize::token::Span;
use crate::lang::Lazy;
use crate::lang::module::ModuleId;

#[derive(Debug)]
pub enum Error {
  Token(tokenize::Error),
  Expected {
    what: &'static str,
    at: Span,
  },
  Invalid {
    what: &'static str,
    at: Span,
  },
}

pub fn asterize<'pool>(lazy: &mut Lazy<'pool>, pool: &'pool StringPool, id: ModuleId) -> Result<(), Error> {
  let path = lazy.get_path(id).to_owned();
  let file = File::open(&path).expect("failed to open path");
  let meta_reader = BufferedUtf8MetadataReader::<64, _>::new(file);
  let tokenizer = Tokenizer::new(pool, id, meta_reader);
  let mut rereader = Rereader::new(tokenizer, id);

  if let Err(err) = make::make(lazy, &mut rereader) {
    let (what, at) = match &err {
      Error::Token(_error) => todo!(),
      Error::Expected { what, at } => (format!("expected {what}"), at),
      Error::Invalid { what, at } => (format!("invalid {what}"), at),
    };

    // Let's do some error printing!
    // Rip the tokens from the tokenizer -- this should drop the file handle
    let mut tokens = rereader.examine_tokens();

    let mut file = File::open(&path)
      .expect("failed to open source for error printing");

    // We're going to show one line before and after the problem.
    // Go to the problem and locate the last newline (or start of file)
    let mut curr_position = file.seek(SeekFrom::Start(at.start.position as _))
      .expect("seek failed");

    // We'll look for the newline in a window of this size, rather than backing
    // up one byte at a time
    const SEEK_WINDOW: u64 = 64;

    let mut newlines_to_find = 2;
    let offset = loop {
      // I don't know what happens if we try to seek past the beginning of the
      // file.  To be safe:
      let window = SEEK_WINDOW.min(curr_position);
      curr_position -= window;

      // Rewind and read the next `window` bytes
      file.seek_relative(-(window as i64)).expect("bad seek");

      let mut bytes = vec![0; window as _];
      file.read_exact(&mut bytes).expect("failed to read bytes from file");

      // Here's where I'd worry about not actually being aligned to UTF-8 bytes,
      // but a newline is just a one-byte character.  Just look for that, since
      // UTF-8 sequences have that high bit set to distinguish them.
      let search_result = {
        bytes.into_iter().enumerate().rev().find_map(|(offset, byte)| {
          if byte == b'\n' {
            newlines_to_find -= 1;

            if newlines_to_find == 0 {
              Some((offset + 1) as u64)
            } else {
              None
            }
          } else {
            None
          }
        })
      };

      if let Some(offset) = search_result {
        break (window - offset) as i64;
      };

      // If we made it all the way to the beginning without finding a newline,
      // then we'll call the beginning the start of our error window:
      // This means, rewind the whole window.  We just read it, now we're at the
      // end.
      if curr_position == 0 {
        break window as i64;
      };

      // Otherwise, back it up some more (continue)
    };

    // Go to that offset
    let curr_position = file.seek(SeekFrom::Current(-offset))
      .expect("bad seek") as usize;

    // Now we have the start of the line that precedes our error
    let initial_bytes_len = at.end.position - curr_position;
    let mut error_source_bytes = vec![0; initial_bytes_len];

    // No BufReader here since the buffer size is 8KiB.  I expect error spans
    // to be significantly less than this amount.  Read from our line before
    // until the end of the error.  After this, we'll read in the last line
    // manually
    file.read_exact(&mut error_source_bytes[1..])
      .expect("failed to read error source");

    let mut newlines_to_find = 2;
    let mut offset = 0;
    // Get that last line.  No newline will be added at the end.
    #[allow(clippy::unbuffered_bytes)]
    let mut bytes = file.bytes();
    loop {
      let ch = match bytes.next() {
        Some(Ok(b'\n')) | None => {
          newlines_to_find -= 1;

          if newlines_to_find == 0 {
            break;
          };

          b'\n'
        },
        Some(Ok(other)) => other,
        Some(err @ Err(_)) => {
          err.expect("failed to read byte-by-byte");
          unreachable!();
        },
      };

      let here = curr_position + offset;
      while tokens.front().is_some_and(|(_, span)| span.start.position < here) {
        if let Some((_, front_span)) = tokens.pop_front() {
          if front_span.end.position == here {
            error_source_bytes.extend_from_slice("\x1b[0m".as_bytes());
          };
        };
      };

      if let Some((front, front_span)) = tokens.front() {
        if front_span.start.position == here {
          let color_code = match front {
            tokenize::token::Token::Identifier(..) => 31,
            tokenize::token::Token::Keyword(..) => 32,
            tokenize::token::Token::Operator(..) => 33,
            tokenize::token::Token::Grouping(..) => 34,
            tokenize::token::Token::Whitespace => 35,
            tokenize::token::Token::Indent(..) => 36,
            tokenize::token::Token::Comment(..) => 37,
            tokenize::token::Token::Numeric { .. } => 38,
          };

          error_source_bytes.extend_from_slice(format!("\x1b[{color_code}m").as_bytes());
        };
      };

      error_source_bytes.push(ch);
      offset += 1;
    };

    let error_source = std::str::from_utf8(&error_source_bytes)
      .expect("source bytes aren't utf8!");

    let expected_line_chars = (at.end.line.ilog10() + 1).max(5) as usize;
    let full_padding = " ".repeat(expected_line_chars);

    println!(
      "\x1b[1m\x1b[31merror\x1b[0m | {what} at \x1b[1m{path}:{line}:{col}\x1b[0m",
      path = path.to_string_lossy(),
      line = at.start.line,
      col = at.start.column,
    );

    let mut current_line = at.start.line - 1;
    for (i, ch) in error_source.chars().enumerate() {
      if i == 0 {
        print!("{full_padding} | ");
      };

      if ch == '\n' {
        println!("\x1b[0m");

        current_line += 1;

        if current_line <= at.end.line {
          print!("\x1b[1m{current_line:>expected_line_chars$}\x1b[0m | ");
        } else {
          print!("{full_padding} | ");
        };
      } else {
        print!("{ch}");
      };
    };

    println!("\x1b[0m");

    return Err(err);
  };

  Ok(())
}
