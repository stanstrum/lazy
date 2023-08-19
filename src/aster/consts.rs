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
