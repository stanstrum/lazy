/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

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
    } else if read::begins_with(reader, consts::keyword::IMPL) {
      NotImplementedSnafu {
        what: "Impl",
        offset: reader.offset()
      }.fail()
    } else {
      UnknownSnafu { what: "Structure", offset: reader.offset() }.fail()
    }
  }
}
