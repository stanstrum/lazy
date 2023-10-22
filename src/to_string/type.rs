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
    Literal::UnicodeString(text)
    | Literal::ByteString(text)
    | Literal::CString(text) => {
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
      Literal::CString(_) => {
        format!("{LIGHT_YELLOW}c\"{}\"{CLEAR}", stringify_string(&self.l))
      },
      Literal::ByteChar(_) =>
        format!("{LIGHT_YELLOW}b\'{}\'{CLEAR}", stringify_string(&self.l)),
      Literal::IntLiteral(text)
      | Literal::FloatLiteral(text) => {
        format!("{MINT}{text}{CLEAR}")
      },
      _ => todo!("exhaustive for literal ast {:#?}", &self.l)
    }
  }
}

impl std::string::ToString for Type {
  fn to_string(&self) -> String {
    match self {
      Type::Intrinsic(intrinsic) => intrinsic.get_name(),
      Type::Function(func) => {
        let func = unsafe { &**func };

        let mut args = func.decl.args.values().collect::<Vec<_>>();
        args.sort_by_key(|arg| arg.span().start);

        let args = args.iter().map(|arg| arg.to_string())
          .collect::<Vec<_>>();

        format!("({}: {}", func.decl.ret.to_string(), args.join(", "))
      },
      Type::Struct(r#struct) => {
        let r#struct = unsafe { &**r#struct };

        r#struct.ident.to_string()
      },
      Type::External(external) => {
        let external = unsafe { &**external };

        let mut args = external.args.values().collect::<Vec<_>>();
        args.sort_by_key(|arg| arg.span().start);

        let args = args.iter().map(|arg| arg.to_string())
          .collect::<Vec<_>>();

        format!("({}: {}", external.ret.to_string(), args.join(", "))
      }
      Type::ConstReferenceTo(ty) => format!("&{}", ty.to_string()),
      Type::MutReferenceTo(ty) => format!("&mut {}", ty.to_string()),
      Type::ConstPtrTo(ty) => format!("*{}", ty.to_string()),
      Type::MutPtrTo(ty) => format!("*mut {}", ty.to_string()),
      Type::ArrayOf(count, ty) => {
        let count = match count {
          Some(count) => count.to_string(),
          None => "".to_owned(),
        };

        format!("[{}]{}", count, ty.to_string())
      },
      Type::Generic(ident, ..) => ident.to_string(),
      Type::Defined(ty) => {
        let ty = unsafe { &**ty };

        ty.to_string()
      },
      Type::Unknown(qual) => {
        format!("{DARK_GRAY}/* unknown */ {}", qual.to_string())
      },
      Type::UnresolvedLiteral(_) => {
        format!("{DARK_GRAY}/* unresolved literal */")
      },
      Type::Unresolved => "/* unresolved */".to_owned()
    }
  }
}

impl std::string::ToString for TypeAST {
  fn to_string(&self) -> String {
    match &self.e {
      Type::Intrinsic(ptr) => {
        format!(
          "{LIGHT_RED}{}{CLEAR}", ptr.get_name()
        )
      },
      Type::ConstReferenceTo(ty) => format!("&{}", ty.to_string()),
      Type::ArrayOf(len, ty) => {
        match len {
          Some(lit) => format!("[{}]{}", lit.to_string(), ty.to_string()),
          None => format!("[]{}", ty.to_string())
        }
      },
      Type::Struct(r#struct) => {
        let r#struct = unsafe { &**r#struct };

        r#struct.ident.to_string()
      },
      Type::Unknown(ident) => {
        format!("{LIGHT_RED}{UNDERLINE}{}{CLEAR}", ident.to_string())
      },
      Type::Defined(defined) => {
        let defined = unsafe { &**defined };

        defined.to_string()
      },
      Type::Unresolved => format!("{LIGHT_RED}/* unresolved */{CLEAR}"),
      _ => todo!("exhaustive typeast: {:#?}", self.e)
    }
  }
}
