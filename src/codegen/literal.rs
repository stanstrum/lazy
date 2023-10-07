/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use inkwell::{
  values::{
    BasicValueEnum, BasicValue
  },
  types::BasicType,
  AddressSpace
};

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
        let i32_type = self.context.i32_type();

        let values = text
          .chars()
          .map(|ch| i32_type.const_int(ch as u64, false))
          .collect::<Vec<_>>();

        let name = self.unique_name("unicode_text");
        let size = text.len() as u32;

        let global = self.module.add_global(
          i32_type.array_type(size)
            .as_basic_type_enum(),
          Some(AddressSpace::default()),
          &name
        );

        let value = i32_type.const_array(values.as_slice());
        global.set_initializer(&value);

        global.as_basic_value_enum()
      },
      Literal::ByteString(text) => {
        let i8_type = self.context.i8_type();

        let values = text
          .chars()
          .map(|ch| i8_type.const_int(ch as u64, false))
          .collect::<Vec<_>>();

        let name = self.unique_name("unicode_text");
        let size = text.len() as u32;

        let global = self.module.add_global(
          i8_type.array_type(size)
            .as_basic_type_enum(),
          Some(AddressSpace::default()),
          &name
        );

        let value = i8_type.const_array(values.as_slice());
        global.set_initializer(&value);

        global.as_basic_value_enum()
      },
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
