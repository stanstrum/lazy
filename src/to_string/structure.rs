/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::aster::ast::*;

use crate::aster::intrinsics;
use crate::colors::*;
use super::{
  str_line_pfx,
  INDENTATION
};

use std::io::Write;

enum NamespaceChild<'a> {
  Import(&'a ImportAST),
  Structure(&'a Structure)
}

impl std::string::ToString for NamespaceChild<'_> {
  fn to_string(&self) -> String {
    match self {
      NamespaceChild::Import(import) => import.to_string(),
      NamespaceChild::Structure(structure) => structure.to_string(),
    }
  }
}

impl<'a> From<&'a ImportAST> for NamespaceChild<'a> {
  fn from(value: &'a ImportAST) -> Self {
    Self::Import(value)
  }
}

impl<'a> From<&'a Structure> for NamespaceChild<'a> {
  fn from(value: &'a Structure) -> Self {
    Self::Structure(value)
  }
}

impl GetSpan for NamespaceChild<'_> {
  fn span(&self) -> Span {
    match self {
      NamespaceChild::Import(import) => import.span(),
      NamespaceChild::Structure(structure) => structure.span(),
    }
  }
}

impl std::string::ToString for ImportPatternAST {
  fn to_string(&self) -> String {
    match self {
      ImportPatternAST::Qualify { ident, child, .. } => {
        format!("{}::{}",
          ident.to_string(),
          child.to_string()
        )
      },
      ImportPatternAST::Brace { children, .. } => {
        let mut text = String::new();

        let children_text: Vec<String> = children.iter()
          .map(|child| child.to_string())
          .collect();

        // let should_do_newlines = children_text.iter().fold(0, |acc, text| text.len()) >= 20
        text += "{\n";

        let len = children_text.len();
        for (i, child_text) in children_text.into_iter().enumerate() {

          text += str_line_pfx(child_text, "  ").as_str();

          if i + 1 != len {
            text += ",";
          };

          text += "\n";
        };

        text += "}";

        text
      },
      ImportPatternAST::Ident { ident, alias, ..  } => {
        if alias.is_some() {
          format!("{} {LIGHT_RED}as{CLEAR} {}",
            ident.to_string(),
            alias.as_ref().unwrap().to_string()
          )
        } else {
          ident.to_string()
        }
      },
    }
  }
}

impl std::string::ToString for ImportAST {
  fn to_string(&self) -> String {
    format!("{LIGHT_RED}import{CLEAR} {} {LIGHT_RED}from{CLEAR} {}",
      self.pattern.to_string().as_str(),
      self.from.to_string().as_str()
    )
  }
}

impl std::string::ToString for NamespaceAST {
  fn to_string(&self) -> String {
    let mut w: Vec<u8> = vec![];

    let imports_iter = self.imports.iter().map(NamespaceChild::from);
    let map_iter = self.map.values().map(NamespaceChild::from);
    let mut collected: Vec<NamespaceChild> = imports_iter.chain(map_iter).collect();

    collected.sort_by_key(
      |x| x.span().start
    );

    for structure in collected {
      if matches!(structure,
        NamespaceChild::Structure(
          | Structure::ImportedNamespace { .. }
          | Structure::ImportedStructure { .. }
        )
      ) {
        continue;
      };

      writeln!(&mut w, "{};", structure.to_string()).unwrap();
      writeln!(&mut w).unwrap();
    };

    let src = String::from_utf8(w)
      .expect("Failed to write buffer to String");

    format!(
      "{LIGHT_RED}namespace{CLEAR} {CREME}{}{CLEAR} {{\n{}\n}}",
      self.ident.to_string(),
      str_line_pfx(src, INDENTATION)
    )
  }
}

impl std::string::ToString for TemplateConstraint {
  fn to_string(&self) -> String {
    match self {
      TemplateConstraint::Unconstrained(ident) => ident.to_string(),
      TemplateConstraint::Extends(_, _) => todo!("to_string for templateconstraint::extends"),
    }
  }
}

impl std::string::ToString for TemplateAST {
  fn to_string(&self) -> String {
    let mut text = format!("{LIGHT_RED}template{CLEAR}: ");

    let (last, rest) = self.constraints.split_last().unwrap();

    for constraint in rest {
      text += format!("{}, ", constraint.to_string()).as_str();
    };

    text += last.to_string().as_str();

    text
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
        ident, members, template, ..
      }) => {
        let mut text = String::new();

        if let Some(template) = template {
          text += template.to_string().as_str();
          text += ";\n";
        };

        text += format!("{LIGHT_RED}struct{CLEAR} {} {{",
          ident.to_string()
        ).as_str();

        if !members.is_empty() {
          text.push('\n');
        };

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
      },
      Structure::ExternDecl(r#extern) => {
        let mut text = format!("{LIGHT_RED}extern{CLEAR} ");

        text += &r#extern.ident.to_string();

        match &r#extern.ret.e {
          Type::Intrinsic(intrinsics::VOID) => {},
          _ => {
            text += &format!(" -> {}", r#extern.ret.to_string());
          }
        };

        let mut args = r#extern.args.iter().collect::<Vec<_>>();
        args.sort_by_key(|(ident, _)| ident.span().start);

        if !args.is_empty() || r#extern.varargs {
          text += ":\n";
        };

        for (i, (ident, ty)) in args.iter().enumerate() {
          let arg_text = format!("{} {}",
            ty.to_string(), ident.to_string()
          );

          text += &str_line_pfx(arg_text, INDENTATION);

          if i + 1 != args.len() || r#extern.varargs {
            text += ",";
            text += "\n";
          };
        };

        if r#extern.varargs {
          text += "  ...";
        };

        text
      },
      Structure::ImportedNamespace { .. }
      | Structure::ImportedStructure { .. } => "".to_string()
    }
  }
}

impl std::string::ToString for FunctionDeclAST {
  fn to_string(&self) -> String {
    let mut w: Vec<u8> = vec![];

    write!(&mut w, "{CREME}{}{CLEAR}", self.ident.to_string()).unwrap();

    match self.ret.e {
      Type::Intrinsic(intrinsics::VOID) => {},
      _ => {
        write!(&mut w, " -> {}", self.ret.to_string()).unwrap();
      }
    };

    if self.args.is_empty() {
      write!(&mut w, " ").unwrap();
    } else {
      writeln!(&mut w, ":").unwrap();

      let last = self.args.len() - 1;

      let mut arg_pairs = self.args.iter().collect::<Vec<_>>();
      arg_pairs.sort_by_key(|(ident, _)| ident.span().start);

      for (i, (ident, ty)) in arg_pairs.iter().enumerate() {
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
