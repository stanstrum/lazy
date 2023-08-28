/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

 use crate::try_make;

use super::super::{
  ast::*,
  SourceReader,
  seek_read::read,
  consts,
  errors::*,
  AsterResult
};

impl Structure {
  pub fn make(reader: &mut SourceReader) -> AsterResult<(String, Self)> {
    if read::begins_with(reader, consts::keyword::FN) {
      let func = FunctionAST::make(reader)?;

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
    } else {
      UnknownSnafu { what: "Structure", offset: reader.offset() }.fail()
    }
  }
}
