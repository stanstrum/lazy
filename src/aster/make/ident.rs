/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use super::super::{
  SourceReader,
  AsterResult,
  errors::*,
  ast::*
};

impl IdentAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    let mut text = String::new();

    for ctr in 0usize.. {
      let Ok(first) = reader.read(1) else {
        return ExpectedSnafu {
          what: "More characters (for Ident)",
          offset: reader.offset()
        }.fail();
      };

      let first = first.chars().nth(0).unwrap();

      match first {
        '_' => {},
        _ if first.is_alphabetic() => {},
        _ if ctr != 0 && first.is_numeric() => {},
        _ if ctr == 0 => {
          reader.rewind(1).unwrap();

          return ExpectedSnafu {
            what: "Ident",
            offset: reader.offset()
          }.fail();
        },
        _ => {
          reader.rewind(1).unwrap();

          break;
        }
      };

      text.push(first);
    };

    let span = reader.span_since(start);

    Ok(Self { span, text })
  }
}
