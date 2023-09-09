/* Copyright (c) 2023, Stan Strum
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

fn stringify_string(lit: &Literal) -> String {
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
          0..=255 => { write!(&mut w, "x{:x<2}", ch as u32).unwrap(); },
          _ => { write!(&mut w, "u{:x<8}", ch as u32).unwrap(); }
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
      Literal::String(_) | Literal::ByteString(_) => stringify_string(&self.l),
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
      AtomExpression::FnCall(callee, args) => {
        let mut w: Vec<u8> = vec![];

        match &**callee {
          FnCallee::Qualified(ident) => {
            write!(&mut w, "{}", ident.to_string()).unwrap();
          },
          FnCallee::SubExpression(SubExpressionAST { e, .. }) => {
            write!(&mut w, "({})", e.to_string()).unwrap();
          },
        };

        write!(&mut w, "(").unwrap();

        if args.len() != 0 {
          let last = args.len() - 1;

          for (i, arg) in args.iter().enumerate() {
            write!(&mut w, "{}", arg.to_string()).unwrap();

            if i != last {
              write!(&mut w, ", ").unwrap();
            };
          };
        };

        write!(&mut w, ")").unwrap();

        String::from_utf8(w).unwrap()
      },
      AtomExpression::Variable(ident) => ident.to_string(),
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

        match r#else {
          Some(r#else) => {
            write!(&mut w, " {LIGHT_RED}else{CLEAR} {}", r#else.to_string()).unwrap();
          },
          _ => {}
        };
      },
      ControlFlow::While(a, b) => {
        write!(&mut w, "{LIGHT_RED}while{CLEAR} {} {}", a.to_string(), b.to_string()).unwrap();
      },
      ControlFlow::DoWhile(a, b) => {
        todo!()
      },
    };

    String::from_utf8(w).unwrap()
  }
}

impl std::string::ToString for OperatorExpressionAST {
  fn to_string(&self) -> String {
    assert!(self.exprs.len() == self.ops.len() + 1);

    let mut w: Vec<u8> = vec![];

    let ops: Vec<Option<&BinaryOperator>> = (0..=self.ops.len()).map(
      |i| self.ops.get(i)
    ).collect();

    for (expr, op) in self.exprs.iter().zip(ops.iter()) {
      write!(&mut w, "{}", expr.to_string()).unwrap();

      if op.is_none() {
        break;
      };

      let op_txt = match op.unwrap() {
        BinaryOperator::Dot => write!(&mut w, "{}", consts::operator::DOT).unwrap(),
        BinaryOperator::DerefDot => write!(&mut w, "{}", consts::operator::DEREF_DOT).unwrap(),
        BinaryOperator::Add => write!(&mut w, " {} ", consts::operator::ADD).unwrap(),
        BinaryOperator::Sub => write!(&mut w, " {} ", consts::operator::SUB).unwrap(),
        BinaryOperator::Mul => write!(&mut w, " {} ", consts::operator::MUL).unwrap(),
        BinaryOperator::Div => write!(&mut w, " {} ", consts::operator::DIV).unwrap(),
        BinaryOperator::Exp => write!(&mut w, " {} ", consts::operator::EXP).unwrap(),
        BinaryOperator::Mod => write!(&mut w, " {} ", consts::operator::MOD).unwrap(),
        BinaryOperator::Equals => write!(&mut w, " {} ", consts::operator::EQUALS).unwrap(),
        BinaryOperator::NotEquals => write!(&mut w, " {} ", consts::operator::NOTEQUALS).unwrap(),
        BinaryOperator::Greater => write!(&mut w, " {} ", consts::operator::GT).unwrap(),
        BinaryOperator::GreaterThanEquals => write!(&mut w, " {} ", consts::operator::GEQ).unwrap(),
        BinaryOperator::LessThan => write!(&mut w, " {} ", consts::operator::LT).unwrap(),
        BinaryOperator::LessThanEquals => write!(&mut w, " {} ", consts::operator::LEQ).unwrap(),
        BinaryOperator::LogicalAnd => write!(&mut w, " {} ", consts::operator::LOGICALAND).unwrap(),
        BinaryOperator::LogicalOr => write!(&mut w, " {} ", consts::operator::LOGICALOR).unwrap(),
        BinaryOperator::LogicalXOR => write!(&mut w, " {} ", consts::operator::LOGICALXOR).unwrap(),
        BinaryOperator::BitAnd => write!(&mut w, " {} ", consts::operator::BITAND).unwrap(),
        BinaryOperator::BitOr => write!(&mut w, " {} ", consts::operator::BITOR).unwrap(),
        BinaryOperator::BitXOR => write!(&mut w, " {} ", consts::operator::BITXOR).unwrap(),
        BinaryOperator::ArithmeticShr => write!(&mut w, " {} ", consts::operator::ASHR).unwrap(),
        BinaryOperator::LogicalShr => write!(&mut w, " {} ", consts::operator::LSHR).unwrap(),
        BinaryOperator::LogicalShl => write!(&mut w, " {} ", consts::operator::LSHL).unwrap(),
        BinaryOperator::Pipe => write!(&mut w, " {} ", consts::operator::PIPE).unwrap(),
        BinaryOperator::Assign => write!(&mut w, " {} ", consts::operator::ASSIGN).unwrap(),
        BinaryOperator::AddAssign => write!(&mut w, " {} ", consts::operator::ADD_ASSIGN).unwrap(),
        BinaryOperator::SubAssign => write!(&mut w, " {} ", consts::operator::SUB_ASSIGN).unwrap(),
        BinaryOperator::MulAssign => write!(&mut w, " {} ", consts::operator::MUL_ASSIGN).unwrap(),
        BinaryOperator::DivAssign => write!(&mut w, " {} ", consts::operator::DIV_ASSIGN).unwrap(),
        BinaryOperator::ExpAssign => write!(&mut w, " {} ", consts::operator::EXP_ASSIGN).unwrap(),
        BinaryOperator::ModAssign => write!(&mut w, " {} ", consts::operator::MOD_ASSIGN).unwrap(),
        BinaryOperator::LogicalAndAssign => write!(&mut w, " {} ", consts::operator::LOGICALAND_ASSIGN).unwrap(),
        BinaryOperator::LogicalOrAssign => write!(&mut w, " {} ", consts::operator::LOGICALOR_ASSIGN).unwrap(),
        BinaryOperator::LogicalXORAssign => write!(&mut w, " {} ", consts::operator::LOGICALXOR_ASSIGN).unwrap(),
        BinaryOperator::BitAndAssign => write!(&mut w, " {} ", consts::operator::BITAND_ASSIGN).unwrap(),
        BinaryOperator::BitOrAssign => write!(&mut w, " {} ", consts::operator::BITOR_ASSIGN).unwrap(),
        BinaryOperator::BitXORAssign => write!(&mut w, " {} ", consts::operator::BITXOR_ASSIGN).unwrap(),
        BinaryOperator::ArithmeticShrAssign => write!(&mut w, " {} ", consts::operator::ASHR_ASSIGN).unwrap(),
        BinaryOperator::LogicalShrAssign => write!(&mut w, " {} ", consts::operator::LSHR_ASSIGN).unwrap(),
        BinaryOperator::LogicalShlAssign => write!(&mut w, " {} ", consts::operator::LSHL_ASSIGN).unwrap(),
        BinaryOperator::AssignPipe => write!(&mut w, " {} ", consts::operator::PIPE_ASSIGN).unwrap(),
      };
    };

    String::from_utf8(w).unwrap()
  }
}

impl std::string::ToString for Expression {
  fn to_string(&self) -> String {
    match self {
      Expression::Atom(a) => a.to_string(),
      Expression::Block(a) => a.to_string(),
      Expression::SubExpression(a) => a.to_string(),
      Expression::ControlFlow(a) => a.to_string(),
      Expression::Operator(a) => a.to_string(),
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

    if self.args.len() == 0 {
      write!(&mut w, " ").unwrap();
    } else {
      write!(&mut w, ":\n").unwrap();

      let last = self.args.len() - 1;

      for (i, arg) in self.args.iter().enumerate() {
        write!(&mut w, "  {} {}", arg.0.to_string(), arg.1.to_string()).unwrap();

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
    write!(&mut w, "{};", self.body.to_string()).unwrap();

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

    write!(&mut w, "}};").unwrap();

    String::from_utf8(w).unwrap()
  }
}

impl std::string::ToString for ImplForAST {
  fn to_string(&self) -> String {
    let mut w: Vec<u8> = vec![];

    writeln!(&mut w, "{LIGHT_RED}impl{CLEAR} {}: {CREME}{}{CLEAR} {{", self.ty.to_string(), self.r#trait.to_string()).unwrap();

    write!(&mut w, "{}", methods_to_string(&self.methods)).unwrap();

    write!(&mut w, "}};").unwrap();

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

    write!(&mut w, "}};").unwrap();

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
      _ => todo!("structure tostring {:#?}", self)
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
      writeln!(&mut w, "{}", structure.to_string()).unwrap();
      writeln!(&mut w).unwrap();
    }

    let src = String::from_utf8(w)
      .expect("Failed to write buffer to String");

    format!(
      "{LIGHT_RED}namespace{CLEAR} {CREME}{}{CLEAR} {{\n{}\n}};",
      self.ident.to_string(),
      str_line_pfx(src, INDENTATION)
    )
  }
}
