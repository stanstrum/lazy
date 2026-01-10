mod meta;

use std::fs::File;
use std::io::{BufReader, Read};
use meta::Metadata;

pub struct Utf8Error;

#[derive(Debug)]
pub struct BufferedUtf8MetadataReader<const N: usize> {
  reader: BufReader<File>,
  caret: usize,
  good_start: usize,
  buf_chars: usize,
  buf_bytes: usize,
  parse_buffer: [u8; N],
  pub meta: Metadata,
}

impl<const N: usize> BufferedUtf8MetadataReader<N> {
  pub fn new(file: File) -> Self {
    let reader = BufReader::new(file);

    Self {
      reader,
      caret: 0,
      good_start: 0,
      buf_chars: 0,
      buf_bytes: 0,
      parse_buffer: [0; N],
      meta: Metadata::new(),
    }
  }

  fn good_string(&self) -> &str {
    let window = &self.parse_buffer[self.good_start..self.buf_bytes];
    unsafe { std::str::from_utf8_unchecked(window) }
  }
}

impl<const N: usize> Iterator for BufferedUtf8MetadataReader<N> {
  type Item = Result<char, Utf8Error>;

  fn next(&mut self) -> Option<Self::Item> {
    // if we're out of characters to deliver
    if self.caret == self.buf_chars {
      // reset our char index
      self.caret = 0;

      // read some more bytes in after the last invalid byte(s)
      let Ok(bytes) = self.reader.read(&mut self.parse_buffer[self.good_start..]) else {
        return Some(Err(Utf8Error));
      };

      // compute the new length of the last invalid byte(s) and the new bytes
      self.buf_bytes = self.good_start + bytes;

      // get the amount of good characters for this window
      self.buf_chars = {
        // parse the window
        match std::str::from_utf8(&self.parse_buffer[..self.buf_bytes]) {
          Ok(str) => {
            // all ok, no bytes to tuck away
            self.good_start = 0;
            // the string chars count
            str.chars().count()
          },
          Err(err) => {
            // there was an error, we're guessing it's a utf8 sequence that got
            // cut short.  find out until when we're good and save the remaining
            // bytes for the next go-around
            let valid_up_to = err.valid_up_to();
            let error_bytes = self.buf_bytes - valid_up_to;

            // temp buffer for would-be overlapping memmoves
            let mut temp = [0; N];

            // copy the remainder bytes to the front of temp
            temp[..error_bytes].copy_from_slice(
              &self.parse_buffer[valid_up_to..self.buf_bytes]);
            // copy the good bytes in after the error bytes
            temp[error_bytes..][..valid_up_to].copy_from_slice(
              &self.parse_buffer[..valid_up_to]);

            // note that we will be skipping the invalid bytes from the
            // beginning
            self.good_start = error_bytes;
            self.parse_buffer.copy_from_slice(&temp);

            // the count of good bytes
            self.good_string().chars().count()
          },
        }
      };
    };

    // if after refilling our buffer there are no characters
    // left to read, we are done
    if self.buf_chars == 0 {
      return None;
    };

    // get the next character
    let ch = self.good_string().chars().nth(self.caret).unwrap();
    // increment the caret
    self.caret += 1;

    // update the metadata with this characer
    self.meta.take(ch);

    // yield this char
    Some(Ok(ch))
  }
}
