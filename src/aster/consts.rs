/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

pub mod keyword {
  pub const FN: &str = "fn";

  pub const MUT: &str = "mut";
}

pub mod punctuation {
  pub const RIGHT_ARROW: &str = "->";
  pub const COLON: &str = ":";
  pub const COMMA: &str = ",";

  pub const SEMICOLON: &str = ";";

  pub const LINE_COMMENT: &str = "//";

  pub const BOLLOCKS: &str = ":=";

  pub const QUOTE: &str = "\"";
  pub const APOSTROPHE: &str = "'";
  pub const BACKSLASH: &str = "\\";

  pub const AMPERSAND: &str = "&";
}

pub mod grouping {
  pub const OPEN_BRACE: &str = "{";
  pub const CLOSE_BRACE: &str = "}";
  pub const OPEN_BRACKET: &str = "[";
  pub const CLOSE_BRACKET: &str = "]";
  pub const OPEN_PARENTHESIS: &str = "(";
  pub const CLOSE_PARENTHESIS: &str = ")";
  pub const OPEN_CHEVRON: &str = "<";
  pub const CLOSE_CHEVRON: &str = ">";

  pub const OPEN_MULTILINE_COMMENT: &str = "/*";
  pub const CLOSE_MULTILINE_COMMENT: &str = "*/";
}

pub mod ascii {
  // we're going oldschool with this :3

  pub const NL: char = '\0';
  pub const BL: char = '\x07';
  pub const BS: char = '\x08';
  pub const HT: char = '\t';
  pub const LF: char = '\n';
  pub const VT: char = '\x0b';
  pub const FF: char = '\x0c';
  pub const CR: char = '\r';
  pub const ES: char = '\x1b';

  pub const NL_ESCAPE: char = '0';
  pub const BL_ESCAPE: char = 'a';
  pub const BS_ESCAPE: char = 'b';
  pub const HT_ESCAPE: char = 't';
  pub const LF_ESCAPE: char = 'n';
  pub const VT_ESCAPE: char = 'v';
  pub const FF_ESCAPE: char = 'f';
  pub const CR_ESCAPE: char = 'r';
  pub const ES_ESCAPE: char = 'e';

  pub const HEX_ESCAPE: char = 'x';
  pub const UNI_ESCAPE: char = 'u';
}
