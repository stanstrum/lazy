/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::{io::{Write, /* Result */}, /* str::FromStr */};
use crate::aster::consts;

use super::ast::*;

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

impl std::string::ToString for Literal {
  fn to_string(&self) -> String {
    match self {
      Literal::String(text) => {
        let mut escaped = String::new();

        for ch in text.chars() {
          match ch {
            '\\' | '"' => { escaped.push('\\'); escaped.push(ch); }
            ' '..='~' => escaped.push(ch),
            consts::ascii::NL => escaped.push_str("\\0"),
            consts::ascii::BL => escaped.push_str("\\a"),
            consts::ascii::BS => escaped.push_str("\\b"),
            consts::ascii::HT => escaped.push_str("\\t"),
            consts::ascii::LF => escaped.push_str("\\n"),
            consts::ascii::VT => escaped.push_str("\\v"),
            consts::ascii::FF => escaped.push_str("\\f"),
            consts::ascii::CR => escaped.push_str("\\r"),
            consts::ascii::ES => escaped.push_str("\\e"),
            _ => {
              escaped.push('\\');

              let s = match ch as u32 {
                0..=255 => format!("{:x<2}", ch as u32),
                _ => todo!()
              };

              escaped.push_str(s.as_str());
            }
          };
        };

        format!("\"{}\"", escaped)
      },
      _ => todo!()
    }
  }
}

impl std::string::ToString for AtomExpressionAST {
  fn to_string(&self) -> String {
    match &self.a {
      AtomExpression::Assignment {
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
          "/* intrinsic */ {}", name
        )
      },
      _ => todo!("exhaustive typeast: {:#?}", self.e)
    }
  }
}

impl std::string::ToString for FunctionAST {
  fn to_string(&self) -> String {
    let mut w: Vec<u8> = vec![];

    if self.args.len() == 0 {
      write!(&mut w, "fn {} -> {} ", self.ident.to_string(), self.ret.to_string()).unwrap();
    } else {
      writeln!(&mut w, "fn {} -> {}:", self.ident.to_string(), self.ret.to_string()).unwrap();
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
    self.text.clone()
  }
}

impl std::string::ToString for NamespaceAST {
  fn to_string(&self) -> String {
    let mut w: Vec<u8> = vec![];

    for (name, structure) in self.map.iter() {
      let span = match structure {
        Structure::FunctionAST(FunctionAST { span, .. }) => span,
        Structure::NamespaceAST(NamespaceAST { span, .. }) => span,
      };

      writeln!(&mut w, "// {} ({}:{})", name, span.start, span.end).unwrap();
      writeln!(&mut w, "{}", structure.to_string()).unwrap();
      writeln!(&mut w).unwrap();
    }

    let src = String::from_utf8(w)
      .expect("Failed to write buffer to String");

    format!(
      "namespace {} {{\n{}\n}}",
      self.ident.to_string(),
      str_line_pfx(src, INDENTATION)
    )
  }
}
