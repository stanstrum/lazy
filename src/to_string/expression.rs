/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::aster::{
  ast::*,
  consts
};

use crate::colors::*;
use super::{
  str_line_pfx,
  INDENTATION
};

use std::io::Write;

// todo: make a way to store the name for this
// for better IR generation ...
impl std::string::ToString for VariableReference {
  fn to_string(&self) -> String {
    match self {
      VariableReference::ResolvedVariable(..) => "variable",
      VariableReference::ResolvedArgument(..) => "argument",
      VariableReference::ResolvedFunction(..) => "function",
      VariableReference::ResolvedMemberFunction(..) => "member function",
      VariableReference::ResolvedMemberOf(..) => "member of",
      VariableReference::ResolvedExternal(..) => "external"
    }.to_string()
  }
}

impl std::string::ToString for FullyQualifiedIdentAST {
  fn to_string(&self) -> String {
    let mut text = self.ident.to_string();

    if let Some(generics) = self.generics.as_ref() {
      text += "<";

      if let Some((first, rest)) = generics.split_first() {
        text += &first.to_string();

        for generic in rest {
          text += &generic.to_string();
        };
      };

      text += ">";
    };

    text
  }
}

impl std::string::ToString for FullyQualifiedAST {
  fn to_string(&self) -> String {
    let (first, parts) = self.parts.split_first().unwrap();
    let mut text = first.to_string();

    for part in parts {
      text += &format!("::{}", part.to_string());
    };

    text
  }
}

impl std::string::ToString for AtomExpressionAST {
  fn to_string(&self) -> String {
    match &self.a {
      AtomExpression::Literal(lit) => lit.to_string(),
      AtomExpression::UnresolvedVariable(qual) => format!("{DARK_GRAY}/* unresolved */{CLEAR} {}", qual.to_string()),
      AtomExpression::ValueVariable(qual, var_ref) => {
        format!("{DARK_GRAY}/* value {} */{CLEAR} {}", var_ref.to_string(), qual.to_string())
      },
      AtomExpression::DestinationVariable(qual, var_ref) => {
        format!("{DARK_GRAY}/* destination {} */{CLEAR} {}", var_ref.to_string(), qual.to_string())
      },
      AtomExpression::Return(expr) => {
        if expr.is_some() {
          format!("{LIGHT_RED}return{CLEAR} {}", expr.as_ref().unwrap().to_string())
        } else {
          "return".to_owned()
        }
      },
      AtomExpression::Break(_) => todo!("atomexpression break"),
      AtomExpression::StructInitializer(StructInitializerAST {
        fqual, members, ..
      }) => {
        let mut text = String::new();

        text += fqual.to_string().as_str();
        text += " {";

        if !members.is_empty() {
          text += "\n";
        };

        for (i, (ident, expr)) in members.iter().enumerate() {
          text += str_line_pfx(
            format!("{}: {}",
              ident.to_string(),
              expr.to_string()
            ),
            "  "
          ).as_str();

          if i + 1 != members.len() {
            text += ",";
          };

          text += "\n";
        };

        text += "}";

        text
      },
      #[allow(unused)]
      other => todo!("to_string atom expression {other:?}")
    }
  }
}

impl std::string::ToString for BlockExpressionChild {
  fn to_string(&self) -> String {
    match self {
      BlockExpressionChild::Binding(BindingAST {
        r#mut, ty, ident, value, ..
      }) => {
        let mut text = String::new();

        if r#mut.is_some() {
          text.push_str(format!("{LIGHT_RED}mut{CLEAR}").as_str());
          text.push(' ');
        };

        if ty.is_some() {
          text.push_str(
            ty.as_ref().unwrap().to_string().as_str()
          );
          text.push(' ');
        };

        text.push_str(ident.to_string().as_str());

        if value.is_some() {
          text.push(' ');
          text.push_str(consts::punctuation::BOLLOCKS);
          text.push(' ');

          text.push_str(
            value.as_ref().unwrap().to_string().as_str()
          );
        };

        text
      },
      BlockExpressionChild::Expression(expr) => expr.to_string(),
    }
  }
}

impl std::string::ToString for BlockExpressionAST {
  fn to_string(&self) -> String {
    let mut w: Vec<u8> = vec![];

    if self.children.is_empty() {
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

impl std::string::ToString for SubExpressionAST {
  fn to_string(&self) -> String {
    format!("({})", self.e.to_string())
  }
}

impl std::string::ToString for ControlFlowAST {
  fn to_string(&self) -> String {
    let mut w: Vec<u8> = vec![];

    match &self.e {
      ControlFlow::If(branches, r#else) => {
        for (i, (cond, block)) in branches.iter().enumerate() {
          if i != 0 {
            write!(&mut w, " {LIGHT_RED}else{CLEAR} ").unwrap();
          };

          write!(&mut w, "{LIGHT_RED}if{CLEAR} {} {}", cond.to_string(), block.to_string()).unwrap();
        };

        if let Some(r#else) = r#else {
          write!(&mut w, " {LIGHT_RED}else{CLEAR} {}", r#else.to_string()).unwrap();
        };
      },
      ControlFlow::While(a, b) => {
        write!(&mut w, "{LIGHT_RED}while{CLEAR} {} {}", a.to_string(), b.to_string()).unwrap();
      },
      ControlFlow::DoWhile(_a, _b) => {
        todo!()
      },
      ControlFlow::Loop(body) => {
        write!(&mut w, "{LIGHT_RED}loop{CLEAR} {}", body.to_string()).unwrap();
      }
    };

    String::from_utf8(w).unwrap()
  }
}
