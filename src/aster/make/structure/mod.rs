/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

mod r#impl;
mod r#trait;
mod function;
mod member_function;
mod namespace;
mod typealias;
mod r#struct;
mod r#extern;
mod import;

use std::collections::HashMap;

use crate::aster::{
  ast::*,
  SourceReader,
  errors::*,
  AsterResult
};

use super::try_make;

impl Structure {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    if let Some(func) = try_make!(FunctionAST::make, reader) {
      Ok(Structure::Function(func))
    } else if let Some(extern_decl) = try_make!(ExternDeclAST::make, reader) {
      Ok(Structure::ExternDecl(extern_decl))
    } else if let Some(r#trait) = try_make!(TraitAST::make, reader) {
      Ok(Structure::Trait(r#trait))
    } else if let Some(r#impl) = try_make!(ImplAST::make, reader) {
      Ok(Structure::Impl(Impl::Impl(r#impl)))
    } else if let Some(impl_for) = try_make!(ImplForAST::make, reader) {
      Ok(Structure::Impl(Impl::ImplFor(impl_for)))
    } else if let Some(ns) = try_make!(NamespaceAST::make, reader) {
      Ok(Structure::Namespace(ns))
    } else if let Some(ty_alias) = try_make!(TypeAliasAST::make, reader) {
      Ok(Structure::TypeAlias(ty_alias))
    } else if let Some(r#struct) = try_make!(StructAST::make, reader) {
      Ok(Structure::Struct(r#struct))
    } else {
      UnknownSnafu {
        what: "Structure",
        offset: reader.offset(),
        path: reader.path.clone()
      }.fail()
    }
  }

  pub fn to_hashable(&self) -> String {
    match self {
      Structure::Function(func) => {
        let ident = &func.decl.ident;

        ident.to_hashable()
      },
      Structure::ExternDecl(extern_decl) => {
        let ident = &extern_decl.ident;

        ident.to_hashable()
      },
      Structure::Trait(r#trait) => {
        let ident = &r#trait.ident;

        r#ident.to_hashable()
      },
      Structure::Impl(Impl::Impl(r#impl)) => {
        let text = format!(
          "impl!{}",
          r#impl.ty.to_hashable()
        );

        text
      },
      Structure::Impl(Impl::ImplFor(impl_for)) => {
        let text = format!("impl!{}!{}",
          impl_for.ty.to_hashable(),
          impl_for.r#trait.to_hashable()
        );

        text
      },
      Structure::Namespace(ns) => {
        let ident = &ns.ident;

        ident.to_hashable()
      },
      Structure::TypeAlias(ty_alias) => {
        let ident = &ty_alias.ident;

        ident.to_hashable()
      },
      Structure::Struct(r#struct) => {
        let ident = &r#struct.ident;

        ident.to_hashable()
      },
      Structure::ImportedNamespace { ident, .. }
      | Structure::ImportedStructure { ident, .. } => {
        ident.to_hashable()
      },
    }
  }
}
