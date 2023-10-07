/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use inkwell::values::{BasicValueEnum, BasicValue};

use crate::aster::ast::{
  LiteralAST, Type, Literal
};

use super::{
  Codegen,
  CodeGenResult, parse_int_literal
};

impl<'a, 'ctx> Codegen<'a, 'ctx> {
  pub fn generate_literal(&mut self, lit: &LiteralAST, ty: &Type) -> CodeGenResult<BasicValueEnum<'ctx>> {
    Ok(match &lit.l {
      Literal::UnicodeString(text) => {
        let ascii_str = text
          .chars()
          .map(
            |ch| u32::to_le_bytes(ch as u32)
              .map(|byte| byte as char)
          )
          .map(Vec::from)
          .map(|vec| vec.iter().collect::<String>())
          .collect::<String>();

        let name = self.unique_name("unicode_text");

        self.builder.build_global_string_ptr(&ascii_str, &name)
          .as_basic_value_enum()
      },
      Literal::ByteString(_) => todo!("generate_literal bytestring"),
      Literal::CString(_) => todo!("generate_literal cstring"),
      Literal::Char(_) => todo!("generate_literal char"),
      Literal::ByteChar(_) => todo!("generate_literal bytechar"),
      Literal::FloatLiteral(_) => todo!("generate_literal floatliteral"),
      Literal::IntLiteral(text) => {
        let value = parse_int_literal(text);

        // what is "sign_extend" here? (the magic `false` at the end of the next line)
        self.generate_type(ty)?
          .to_basic_metadata()
          .into_int_type()
          .const_int(value, false)
          .as_basic_value_enum()
      },
    })
  }
}
