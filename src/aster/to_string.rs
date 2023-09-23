/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::{io::{Write, /* Result */}, /* str::FromStr */};
use crate::aster::consts;

use super::{ast::*, intrinsics};

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

fn stringify_char(ch: char) -> String {
  match ch {
    '\\' | '"' => { format!("\\{ch}") },
    ' '..='~' => { format!("{}", ch) },
    consts::ascii::NL => { "\\0".to_owned() },
    consts::ascii::BL => { "\\a".to_owned() },
    consts::ascii::BS => { "\\b".to_owned() },
    consts::ascii::HT => { "\\t".to_owned() },
    consts::ascii::LF => { "\\n".to_owned() },
    consts::ascii::VT => { "\\v".to_owned() },
    consts::ascii::FF => { "\\f".to_owned() },
    consts::ascii::CR => { "\\r".to_owned() },
    consts::ascii::ES => { "\\e".to_owned() },
    _ => {
      match ch as u32 {
        0..=255 => { format!("\\x{:x<2}", ch as u32) },
        _ => { format!("\\u{:x<8}", ch as u32) }
      }
    }
  }
}

fn stringify_string(lit: &Literal) -> String {
 match lit {
    Literal::UnicodeString(text) | Literal::ByteString(text) => {
      text
        .chars()
        .map(stringify_char)
        .collect::<String>()
    },
    Literal::ByteChar(ch) => {
      stringify_char(*ch)
    },
    _ => todo!("exhaustive for literal ast: {:#?}", lit)
  }
}

impl std::string::ToString for LiteralAST {
  fn to_string(&self) -> String {
    match &self.l {
      Literal::UnicodeString(_) =>
        format!("{LIGHT_YELLOW}\"{}\"{CLEAR}", stringify_string(&self.l)),
      Literal::ByteString(_) =>
        format!("{LIGHT_YELLOW}b\"{}\"{CLEAR}", stringify_string(&self.l)),
      Literal::ByteChar(_) =>
        format!("{LIGHT_YELLOW}b\'{}\'{CLEAR}", stringify_string(&self.l)),
      Literal::NumericLiteral(s) => {
        format!("{MINT}{s}{CLEAR}")
      },
      _ => todo!("exhaustive for literal ast {:#?}", &self.l)
    }
  }
}

impl std::string::ToString for AtomExpressionAST {
  fn to_string(&self) -> String {
    match &self.a {
      AtomExpression::Literal(lit) => lit.to_string(),
      AtomExpression::Variable(ident, _) => ident.to_string(),
      AtomExpression::Return(expr) => {
        if expr.is_some() {
          format!("{LIGHT_RED}return{CLEAR} {}", expr.as_ref().unwrap().to_string())
        } else {
          "return".to_owned()
        }
      },
      AtomExpression::Break(_) => todo!("atomexpression break"),
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

impl std::string::ToString for BinaryOperator {
  fn to_string(&self) -> String {
    consts::operator::BIN_MAP
      .into_iter()
      .find_map(
        |(key, val)|
          if val == self {
            Some(key)
          } else {
            None
          }
      ).unwrap_or_else(|| panic!("no operator for variant {:#?}", self)).to_string()
  }
}

impl std::string::ToString for UnaryPfxOperator {
  fn to_string(&self) -> String {
    consts::operator::UNARY_PFX_MAP
      .into_iter()
      .find_map(
        |(key, val)|
          if val == self {
            Some(key)
          } else {
            None
          }
      ).unwrap_or_else(|| panic!("no operator for variant {:#?}", self)).to_string()
  }
}

impl std::string::ToString for UnarySfxOperator {
  fn to_string(&self) -> String {
    consts::operator::UNARY_SFX_MAP
      .into_iter()
      .find_map(
        |(key, val)|
          if val == self {
            Some(key)
          } else {
            None
          }
      ).unwrap_or_else(|| panic!("no operator for variant {:#?}", self)).to_string()
  }
}

impl std::string::ToString for UnaryOperator {
  fn to_string(&self) -> String {
    match self {
      UnaryOperator::UnaryPfx(pfx) => pfx.to_string(),
      UnaryOperator::UnarySfx(sfx) => sfx.to_string(),
    }
  }
}

impl std::string::ToString for Operator {
  fn to_string(&self) -> String {
    match self {
      Operator::UnaryPfx(op) => op.to_string(),
      Operator::UnarySfx(op) => op.to_string(),
      Operator::Binary(op) => op.to_string(),
    }
  }
}

impl std::string::ToString for Expression {
  fn to_string(&self) -> String {
    match self {
      Expression::Atom(a) => a.to_string(),
      Expression::Block(a) => a.to_string(),
      Expression::SubExpression(a) => a.to_string(),
      Expression::ControlFlow(a) => a.to_string(),
      Expression::BinaryOperator(BinaryOperatorExpressionAST { a, b, op, .. }) => {
        match op {
          BinaryOperator::Dot | BinaryOperator::DerefDot =>
            format!(
              "{DARK_GRAY}({CLEAR}{}{TEAL}{}{CLEAR}{}{DARK_GRAY}){CLEAR}",
              a.to_string(),
              op.to_string(),
              b.to_string()
            ),
          _ => format!(
            "{DARK_GRAY}({CLEAR}{} {TEAL}{}{CLEAR} {}{DARK_GRAY}){CLEAR}",
            a.to_string(),
            op.to_string(),
            b.to_string()
          )
        }
      },
      Expression::UnaryOperator(UnaryOperatorExpressionAST { expr, op, ..}) => {
        match op {
          UnaryOperator::UnarySfx(UnarySfxOperator::Subscript { arg }) => {
            format!("{}[{}]", expr.to_string(), arg.to_string())
          },
          UnaryOperator::UnarySfx(UnarySfxOperator::Call { args }) => {
            format!(
              "{DARK_GRAY}({CLEAR}{}({}){DARK_GRAY}){CLEAR}",
              expr.to_string(),
              args.iter().map(|arg| arg.to_string()).collect::<Vec<String>>().join(", ")
            )
          },
          UnaryOperator::UnarySfx(_) => {
            format!("{DARK_GRAY}({CLEAR}{}{TEAL}{}{DARK_GRAY}){CLEAR}", expr.to_string(), op.to_string())
          },
          UnaryOperator::UnaryPfx(UnaryPfxOperator::MutRef) => {
            format!("{DARK_GRAY}({CLEAR}{TEAL}{}{CLEAR} {}{DARK_GRAY}){CLEAR}", op.to_string(), expr.to_string())
          },
          UnaryOperator::UnaryPfx(_) => {
            format!("{DARK_GRAY}({CLEAR}{TEAL}{}{CLEAR}{}{DARK_GRAY}){CLEAR}", op.to_string(), expr.to_string())
          },
        }
      },
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
        format!("{DARK_GRAY}/* unknown */{CLEAR} {LIGHT_RED}{UNDERLINE}{}{CLEAR}", ident.to_string())
      },
      _ => todo!("exhaustive typeast: {:#?}", self.e)
    }
  }
}

impl std::string::ToString for FunctionDeclAST {
  fn to_string(&self) -> String {
    let mut w: Vec<u8> = vec![];

    write!(&mut w, "{CREME}{}{CLEAR}", self.ident.to_string()).unwrap();

    match self.ret.e {
      Type::Intrinsic(ptr) if ptr == &intrinsics::VOID => {},
      _ => {
        write!(&mut w, " -> {}", self.ret.to_string()).unwrap();
      }
    };

    if self.args.is_empty() {
      write!(&mut w, " ").unwrap();
    } else {
      writeln!(&mut w, ":").unwrap();

      let last = self.args.len() - 1;

      for (i, (ident, ty)) in self.args.iter().enumerate() {
        write!(&mut w, "  {} {}", ty.to_string(), ident.to_string()).unwrap();

        if i != last {
          write!(&mut w, ",").unwrap();
        };

        writeln!(&mut w).unwrap();
      };
    };

    String::from_utf8(w).unwrap()
  }
}

impl std::string::ToString for FunctionAST {
  fn to_string(&self) -> String {
    let mut w: Vec<u8> = vec![];

    write!(&mut w, "{}", self.decl.to_string()).unwrap();
    write!(&mut w, "{}", self.body.to_string()).unwrap();

    String::from_utf8(w)
      .expect("Failed to write buffer to String")
  }
}

impl std::string::ToString for MemberFunctionDeclAST {
  fn to_string(&self) -> String {
    let mut w: Vec<u8> = vec![];

    if self.public.is_some() {
      write!(&mut w, "{LIGHT_RED}pub{CLEAR} ").unwrap();
    };

    if self.r#static.is_some() {
      write!(&mut w, "{LIGHT_RED}static{CLEAR} ").unwrap();
    };

    if self.r#mut.is_some() {
      write!(&mut w, "{LIGHT_RED}mut{CLEAR} ").unwrap();
    };

    write!(&mut w, "{}", self.decl.to_string()).unwrap();

    String::from_utf8(w).unwrap()
  }
}

impl std::string::ToString for MemberFunctionAST {
  fn to_string(&self) -> String {
    format!("{}{};", self.decl.to_string(), self.body.to_string())
  }
}

fn methods_to_string(methods: &Vec<MemberFunctionAST>) -> String {
  let mut w: Vec<u8> = vec![];

  for (i, method) in methods.iter().enumerate() {
    writeln!(&mut w, "{}",
      str_line_pfx(
        method.to_string(),
        "  "
      )
    ).unwrap();

    if i != methods.len() - 1 {
      writeln!(&mut w).unwrap();
    };
  };

  String::from_utf8(w).unwrap()
}

impl std::string::ToString for ImplAST {
  fn to_string(&self) -> String {
    let mut w: Vec<u8> = vec![];

    writeln!(&mut w, "{LIGHT_RED}impl{CLEAR} {CREME}{}{CLEAR} {{", self.ty.to_string()).unwrap();

    write!(&mut w, "{}", methods_to_string(&self.methods)).unwrap();

    write!(&mut w, "}}").unwrap();

    String::from_utf8(w).unwrap()
  }
}

impl std::string::ToString for ImplForAST {
  fn to_string(&self) -> String {
    let mut w: Vec<u8> = vec![];

    writeln!(&mut w, "{LIGHT_RED}impl{CLEAR} {}: {CREME}{}{CLEAR} {{", self.ty.to_string(), self.r#trait.to_string()).unwrap();

    write!(&mut w, "{}", methods_to_string(&self.methods)).unwrap();

    write!(&mut w, "}}").unwrap();

    String::from_utf8(w).unwrap()
  }
}

impl std::string::ToString for TraitAST {
  fn to_string(&self) -> String {
    let mut w: Vec<u8> = vec![];

    writeln!(&mut w, "{LIGHT_RED}trait{CLEAR} {CREME}{}{CLEAR} {{", self.ident.to_string()).unwrap();

    for (i, decl) in self.decls.iter().enumerate() {
      writeln!(&mut w, "{};",
        str_line_pfx(
          decl.to_string().trim_end().to_string(),
          "  "
        )
      ).unwrap();

      if i != self.decls.len() - 1 {
        writeln!(&mut w).unwrap();
      };
    };

    write!(&mut w, "}}").unwrap();

    String::from_utf8(w).unwrap()
  }
}

impl std::string::ToString for Structure {
  fn to_string(&self) -> String {
    match self {
      Structure::Namespace(ns) => ns.to_string(),
      Structure::Function(func) => func.to_string(),
      Structure::Trait(r#trait) => r#trait.to_string(),
      Structure::Impl(Impl::Impl(r#impl)) => r#impl.to_string(),
      Structure::Impl(Impl::ImplFor(impl_for)) => impl_for.to_string(),
      Structure::TypeAlias(TypeAliasAST {
        ident, ty, ..
      }) => format!("{LIGHT_RED}type{CLEAR} {} := {}",
        ident.to_string(),
        ty.to_string()
      ),
      Structure::Struct(StructAST {
        ident, members, ..
      }) => {
        let mut text = format!("{LIGHT_RED}struct{CLEAR} {} {{\n",
          ident.to_string()
        );

        for (i, (ty, ident)) in members.iter().enumerate() {
          text.push_str(format!("  {} {}",
            ty.to_string(),
            ident.to_string()
          ).as_str());

          if i != members.len() - 1 {
            text.push(',');
          };

          text.push('\n');
        };

        text.push('}');

        text
      }
    }
  }
}

impl std::string::ToString for QualifiedAST {
  fn to_string(&self) -> String {
    let mut w: Vec<u8> = vec![];

    // will always have at least 1
    let last = self.parts.len() - 1;

    for (i, part) in self.parts.iter().enumerate() {
      write!(&mut w, "{}", part.to_string()).unwrap();

      if i != last {
        write!(&mut w, "::").unwrap();
      };
    };

    String::from_utf8(w).unwrap()
  }
}

impl std::string::ToString for IdentAST {
  fn to_string(&self) -> String {
    self.text.to_owned()
  }
}

impl std::string::ToString for NamespaceAST {
  fn to_string(&self) -> String {
    let mut w: Vec<u8> = vec![];

    let mut collected: Vec<(&String, &Structure)> = self.map.iter().collect();
    collected.sort_by(
      |(_, a), (_, b)| a.span().start.cmp(&b.span().start)
    );

    for (name, structure) in collected {
      let span = structure.span();

      writeln!(&mut w, "{DARK_GRAY}// {} ({}:{}){CLEAR}", name, span.start, span.end).unwrap();
      writeln!(&mut w, "{};", structure.to_string()).unwrap();
      writeln!(&mut w).unwrap();
    }

    let src = String::from_utf8(w)
      .expect("Failed to write buffer to String");

    format!(
      "{LIGHT_RED}namespace{CLEAR} {CREME}{}{CLEAR} {{\n{}\n}}",
      self.ident.to_string(),
      str_line_pfx(src, INDENTATION)
    )
  }
}
