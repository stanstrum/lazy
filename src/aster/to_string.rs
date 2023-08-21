/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::{io::{Write, /* Result */}, /* str::FromStr */};
use crate::aster::consts;

use super::{ast::*, formatting::format_message};

use crate::colors::*;

const INDENTATION: &str = "  ";

pub fn str_line_pfx(string: String, pfx: &str) -> String {
  let mut new_string = String::new();

  for line in string.split('\n') {
    if !new_string.is_empty() {
      new_string.push('\n');
    };

    if line.is_empty() {
      continue;
    };

    new_string.push_str(pfx);
    new_string.push_str(line);
  };

  new_string.trim_end().into()
}

fn stringfiy_string(lit: &Literal) -> String {
  let mut w: Vec<u8> = vec![];

  let text = match lit {
    Literal::String(text) => text,
    Literal::ByteString(text) => {
      write!(&mut w, "b").unwrap();

      text
    },
    _ => todo!("exhaustive for literal ast: {:#?}", lit)
  };

  write!(&mut w, "{LIGHT_YELLOW}\"").unwrap();

  for ch in text.chars() {
    match ch {
      '\\' | '"' => { write!(&mut w, "\\{ch}").unwrap(); }
      ' '..='~' => { write!(&mut w, "{}", ch).unwrap(); },
      consts::ascii::NL => { write!(&mut w, "\\0").unwrap(); },
      consts::ascii::BL => { write!(&mut w, "\\a").unwrap(); },
      consts::ascii::BS => { write!(&mut w, "\\b").unwrap(); },
      consts::ascii::HT => { write!(&mut w, "\\t").unwrap(); },
      consts::ascii::LF => { write!(&mut w, "\\n").unwrap(); },
      consts::ascii::VT => { write!(&mut w, "\\v").unwrap(); },
      consts::ascii::FF => { write!(&mut w, "\\f").unwrap(); },
      consts::ascii::CR => { write!(&mut w, "\\r").unwrap(); },
      consts::ascii::ES => { write!(&mut w, "\\e").unwrap(); },
      _ => {
        write!(&mut w, "\\").unwrap();

        match ch as u32 {
          0..=255 => { write!(&mut w, "{:x<2}", ch as u32).unwrap(); },
          _ => todo!()
        };
      }
    };
  };

  write!(&mut w, "\"{CLEAR}").unwrap();

  String::from_utf8(w).unwrap()
}

impl std::string::ToString for LiteralAST {
  fn to_string(&self) -> String {
    match &self.l {
      Literal::String(_) | Literal::ByteString(_) => stringfiy_string(&self.l),
      Literal::NumericLiteral(s) => {
        s.clone()
      },
      _ => todo!("exhaustive for literal ast {:#?}", &self.l)
    }
  }
}

impl std::string::ToString for AtomExpressionAST {
  fn to_string(&self) -> String {
    match &self.a {
      AtomExpression::Binding {
        ty, ident, value
      } => {
        match ty {
          Some(ty) => {
            format!(
              "{} {} := {}",
              ty.to_string(),
              ident.to_string(),
              value.to_string()
            )
          },
          None => {
            format!(
              "{} := {}",
              ident.to_string(),
              value.to_string()
            )
          },
        }
      },
      AtomExpression::Literal(lit) => lit.to_string(),
    }
  }
}

impl std::string::ToString for BlockExpressionAST {
  fn to_string(&self) -> String {
    let mut w: Vec<u8> = vec![];

    if self.children.len() == 0 {
      return "{}".into();
    };

    let last = self.children.len() - 1;
    for (i, child) in self.children.iter().enumerate() {
      write!(&mut w, "{}", child.to_string()).unwrap();

      if !{i == last && self.returns_last} {
        write!(&mut w, ";").unwrap();
      }

      writeln!(&mut w).unwrap();
    };

    let s = String::from_utf8(w)
      .expect("Failed to write buffer to String");

    format!("{{\n{}\n}}", str_line_pfx(s, INDENTATION))
  }
}

impl std::string::ToString for Expression {
  fn to_string(&self) -> String {
    match self {
      Expression::Atom(a) => a.to_string(),
      Expression::Block(a) => a.to_string(),
    }
  }
}

impl std::string::ToString for TypeAST {
  fn to_string(&self) -> String {
    match self.e {
      Type::Intrinsic(ptr) => {
        let name = unsafe { (*ptr).name };

        format!(
          "{LIGHT_RED}{}{CLEAR}", name
        )
      },
      Type::ConstReferenceTo(ref ty) => format!("&{}", ty.to_string()),
      Type::ArrayOf(ref len, ref ty) => {
        match len {
          Some(ref lit) => format!("[{}]{}", lit.to_string(), ty.to_string()),
          None => format!("[]{}", ty.to_string())
        }
      },
      Type::Unknown(ref ident) => {
        format!("{DARK_GRAY}/* unknown */{CLEAR} {}", ident.to_string())
      },
      _ => todo!("exhaustive typeast: {:#?}", self.e)
    }
  }
}

impl std::string::ToString for FunctionAST {
  fn to_string(&self) -> String {
    let mut w: Vec<u8> = vec![];

    if self.args.len() == 0 {
      write!(&mut w, "{LIGHT_RED}fn{CLEAR} {} -> {} ", self.ident.to_string(), self.ret.to_string()).unwrap();
    } else {
      writeln!(&mut w, "{LIGHT_RED}fn{CLEAR} {} -> {}:", self.ident.to_string(), self.ret.to_string()).unwrap();
    };

    for arg in self.args.iter() {
      writeln!(&mut w, "  {} {},", arg.0.to_string(), arg.1.to_string()).unwrap();
    };

    writeln!(&mut w, "{}", self.body.to_string()).unwrap();

    String::from_utf8(w)
      .expect("Failed to write buffer to String")
  }
}

impl std::string::ToString for Structure {
  fn to_string(&self) -> String {
    match self {
      Structure::NamespaceAST(ns) => ns.to_string(),
      Structure::FunctionAST(func) => func.to_string()
    }
  }
}

impl std::string::ToString for IdentAST {
  fn to_string(&self) -> String {
    format!("{LIGHT_GRAY}{BOLD}{}{CLEAR}", self.text)
  }
}

impl std::string::ToString for NamespaceAST {
  fn to_string(&self) -> String {
    let mut w: Vec<u8> = vec![];

    for (name, structure) in self.map.iter() {
      let span = structure.span();

      writeln!(&mut w, "{DARK_GRAY}// {} ({}:{}){CLEAR}", name, span.start, span.end).unwrap();
      writeln!(&mut w, "{}", structure.to_string()).unwrap();
      writeln!(&mut w).unwrap();
    }

    let src = String::from_utf8(w)
      .expect("Failed to write buffer to String");

    format!(
      "{LIGHT_RED}namespace{CLEAR} {} {{\n{}\n}}",
      self.ident.to_string(),
      str_line_pfx(src, INDENTATION)
    )
  }
}
