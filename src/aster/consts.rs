/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

#![allow(unused)]

pub mod keyword {
  pub const FN: &str = "fn";

  pub const TYPE: &str = "type";
  pub const STRUCT: &str = "struct";
  pub const INTERFACE: &str = "interface";

  pub const TRAIT: &str = "trait";

  pub const IMPL: &str = "impl";
  pub const IMPLEMENTS: &str = "implements";
  pub const EXTENDS: &str = "extends";

  pub const NAMESPACE: &str = "namespace";

  pub const PUB: &str = "pub";
  pub const STATIC: &str = "static";

  pub const MUT: &str = "mut";

  pub const INFER: &str = "infer";

  pub const IMPORT: &str = "import";
  pub const FROM: &str = "from";
  pub const EXPORT: &str = "export";

  pub const IF: &str = "if";
  pub const ELSE: &str = "else";
  pub const DO: &str = "do";
  pub const WHILE: &str = "while";
  pub const LOOP: &str = "loop";
  pub const FOR: &str = "for";

  pub const RETURN: &str = "return";
  pub const BREAK: &str = "break";
}

pub mod punctuation {
  pub const RIGHT_ARROW: &str = "->";
  pub const COLON: &str = ":";
  pub const COMMA: &str = ",";

  pub const SEMICOLON: &str = ";";

  pub const DOUBLE_COLON: &str = "::";

  pub const LINE_COMMENT: &str = "//";

  pub const BOLLOCKS: &str = ":=";

  pub const QUOTE: &str = "\"";
  pub const APOSTROPHE: &str = "'";
  pub const BACKSLASH: &str = "\\";

  pub const AMPERSAND: &str = "&";

  pub const HEX_PFX: &str = "0x";
  pub const BIN_PFX: &str = "0b";
  pub const OCT_PFX: &str = "0o";

  pub const U8_SFX: &str = "u8";
  pub const U16_SFX: &str = "u16";
  pub const U32_SFX: &str = "u32";
  pub const U64_SFX: &str = "u64";
  pub const U128_SFX: &str = "u128";
  pub const USIZE_SFX: &str = "usize";
  pub const I8_SFX: &str = "i8";
  pub const I16_SFX: &str = "i16";
  pub const I32_SFX: &str = "i32";
  pub const I64_SFX: &str = "i64";
  pub const I128_SFX: &str = "i128";
  pub const ISIZE_SFX: &str = "isize";
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

use crate::aster::ast::{
  UnaryPfxOperator,
  UnarySfxOperator,
  BinaryOperator,
};

pub mod operator {
  use super::*;
  use phf::phf_ordered_map;

  pub static BIN_MAP: phf::OrderedMap<&'static str, BinaryOperator> = phf_ordered_map! {
    "+=" => BinaryOperator::AddAssign,
    "-=" => BinaryOperator::SubAssign,
    "*=" => BinaryOperator::MulAssign,
    "/=" => BinaryOperator::DivAssign,
    "**=" => BinaryOperator::ExpAssign,
    "%=" => BinaryOperator::ModAssign,
    "&&=" => BinaryOperator::LogicalAndAssign,
    "||=" => BinaryOperator::LogicalOrAssign,
    "^^=" => BinaryOperator::LogicalXORAssign,
    "&=" => BinaryOperator::BitAndAssign,
    "|=" => BinaryOperator::BitOrAssign,
    "^=" => BinaryOperator::BitXORAssign,
    ">>=" => BinaryOperator::ArithmeticShrAssign,
    ">>>=" => BinaryOperator::LogicalShrAssign,
    "<<=" => BinaryOperator::LogicalShlAssign,
    "|>=" => BinaryOperator::AssignPipe,
    "." => BinaryOperator::Dot,
    "->" => BinaryOperator::DerefDot,
    "+" => BinaryOperator::Add,
    "-" => BinaryOperator::Sub,
    "**" => BinaryOperator::Exp,
    "*" => BinaryOperator::Mul,
    "/" => BinaryOperator::Div,
    "%" => BinaryOperator::Mod,
    "==" => BinaryOperator::Equals,
    "!=" => BinaryOperator::NotEquals,
    ">" => BinaryOperator::Greater,
    ">=" => BinaryOperator::GreaterThanEquals,
    "<" => BinaryOperator::LessThan,
    "<=" => BinaryOperator::LessThanEquals,
    "&&" => BinaryOperator::LogicalAnd,
    "||" => BinaryOperator::LogicalOr,
    "^^" => BinaryOperator::LogicalXOR,
    "&" => BinaryOperator::BitAnd,
    "|" => BinaryOperator::BitOr,
    "^" => BinaryOperator::BitXOR,
    ">>" => BinaryOperator::ArithmeticShr,
    ">>>" => BinaryOperator::LogicalShr,
    "<<" => BinaryOperator::LogicalShl,
    "|>" => BinaryOperator::Pipe,
    "=" => BinaryOperator::Assign
  };

  pub static UNARY_PFX_MAP: phf::OrderedMap<&'static str, UnaryPfxOperator> = phf_ordered_map! {
    "&mut" => UnaryPfxOperator::MutRef,
    "&" => UnaryPfxOperator::Ref,
    "*" => UnaryPfxOperator::Deref,
    "!" => UnaryPfxOperator::Not,
    "-" => UnaryPfxOperator::Neg,
    "+" => UnaryPfxOperator::NotNeg,
    "~" => UnaryPfxOperator::BitInvert,
    "++" => UnaryPfxOperator::PreIncrement,
    "--" => UnaryPfxOperator::PreDecrement,
  };

  pub static UNARY_SFX_MAP: phf::OrderedMap<&'static str, UnarySfxOperator> = phf_ordered_map! {
    "++" => UnarySfxOperator::PostIncrement,
    "--" => UnarySfxOperator::PostDecrement,
    // these work differently ...
    // "[...]" => UnarySfxOperator::Subscript,
    // "(...)" => UnarySfxOperator::Call
  };
}
