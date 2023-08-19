/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use snafu::prelude::*;

use super::ast::Span;
pub mod formatting;

#[derive(Debug, Snafu)]
pub enum SourceReaderError {
  #[snafu(display("Invalid seek or rewind"))]
  InvalidSeekRewind,
}

pub struct SourceReader<'a> {
  src: &'a String,
  offset: usize
}


impl<'a> SourceReader<'a> {
  pub fn new(src: &'a String) -> Self {
    Self { src, offset: 0 }
  }

  // pub fn src(&self) -> &'a String {
  //   &self.src
  // }

  pub fn offset(&self) -> usize {
    self.offset
  }

  pub fn len(&self) -> usize {
    self.src.len()
  }

  pub fn remaining(&self) -> usize {
    self.src.len() - self.offset
  }

  pub fn peek(&self, len: usize) -> Option<&'a str> {
    if self.remaining() >= len {
      Some(
        &self.src[self.offset..][..len]
      )
    } else {
      None
    }
  }

  pub fn seek(&mut self, len: usize) -> Result<(), SourceReaderError> {
    if self.remaining() >= len {
      self.offset += len;

      Ok(())
    } else {
      InvalidSeekRewindSnafu.fail()
    }
  }

  pub fn rewind(&mut self, len: usize) -> Result<(), SourceReaderError> {
    if self.offset >= len {
      self.offset -= len;

      Ok(())
    } else {
      InvalidSeekRewindSnafu.fail()
    }
  }

  pub fn read_ch(&mut self) -> Result<char, SourceReaderError> {
    if self.remaining() >= 1 {
      let ch = self.src.chars().nth(self.offset()).unwrap();

      self.offset += 1;

      Ok(ch)
    } else {
      InvalidSeekRewindSnafu.fail()
    }
  }

  pub fn read(&mut self, len: usize) -> Result<&'a str, SourceReaderError> {
    let Some(text) = self.peek(len) else {
      return InvalidSeekRewindSnafu.fail();
    };

    self.seek(len).unwrap();

    Ok(text)
  }
}
