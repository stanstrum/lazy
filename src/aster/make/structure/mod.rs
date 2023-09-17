/* Copyright (c) 2023, Stan Strum
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

use crate::aster::{
  ast::*,
  SourceReader,
  seek_read::read,
  consts,
  errors::*,
  AsterResult
};

use super::try_make;

impl Structure {
  pub fn make(reader: &mut SourceReader) -> AsterResult<(String, Self)> {
    if let Some(func) = try_make!(FunctionAST::make, reader) {
      Ok((
        func.decl.ident.text.to_owned(),
        Structure::Function(func)
      ))
    } else if read::begins_with(reader, consts::keyword::TRAIT) {
      let r#trait = TraitAST::make(reader)?;

      Ok((
        r#trait.ident.text.to_owned(),
        Structure::Trait(r#trait)
      ))
    } else if let Some(r#impl) = try_make!(ImplAST::make, reader) {
      Ok((
        format!(
          "impl!{}",
          r#impl.ty.to_hashable()
        ),
        Structure::Impl(Impl::Impl(r#impl))
      ))
    } else if let Some(impl_for) = try_make!(ImplForAST::make, reader) {
      Ok((
        format!("impl!{}!{}", impl_for.ty.to_hashable(), impl_for.r#trait.to_hashable()),
        Structure::Impl(Impl::ImplFor(impl_for))
      ))
    } else if let Some(ns) = try_make!(NamespaceAST::make, reader) {
      Ok((
        ns.ident.text.to_owned(),
        Structure::Namespace(ns)
      ))
    } else if let Some(ty_alias) = try_make!(TypeAliasAST::make, reader) {
      Ok((
        ty_alias.ident.text.to_owned(),
        Structure::TypeAlias(ty_alias)
      ))
    } else {
      UnknownSnafu { what: "Structure", offset: reader.offset() }.fail()
    }
  }
}
