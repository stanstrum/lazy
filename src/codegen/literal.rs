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

fn is_pointer(ty: &Type) -> bool {
  matches!(ty,
    | Type::ConstReferenceTo(_)
    | Type::MutReferenceTo(_)
    | Type::ConstPtrTo(_)
    | Type::MutPtrTo(_)
  )
}

impl<'a, 'ctx> Codegen<'a, 'ctx> {
  pub fn generate_literal(&mut self, lit: &LiteralAST, ty: &Type) -> CodeGenResult<BasicValueEnum<'ctx>> {
    Ok(match &lit.l {
      Literal::UnicodeString(text) => {
        let i32_type = self.context.i32_type();

        let values = text
          .chars()
          .map(|ch| i32_type.const_int(ch as u64, false))
          .collect::<Vec<_>>();

        let size = text.len() as u32;

        let global = self.module.add_global(
          i32_type.array_type(size)
            .as_basic_type_enum(),
          Some(AddressSpace::default()),
          "unicode_text"
        );

        let value = i32_type.const_array(values.as_slice());
        global.set_initializer(&value);

        let ptr = global.as_basic_value_enum();
        let casted_ptr_ty = i32_type.ptr_type(AddressSpace::default());

        let casted_ptr = self.builder.build_bitcast(
          ptr,
          casted_ptr_ty,
          "unicode_text_ptr_cast"
        );

        let char_slice_ty = self.context.struct_type(&[
          casted_ptr_ty.as_basic_type_enum(),
          self.context.i64_type().as_basic_type_enum(),
        ], false);

        let char_slice = char_slice_ty.const_named_struct(&[
          casted_ptr,
          self.context.i64_type().const_int(size as u64, false).as_basic_value_enum()
        ]);

        char_slice.as_basic_value_enum()
      },
      Literal::ByteString(text) => {
        let i8_type = self.context.i8_type();

        let values = text
          .chars()
          .map(|ch| i8_type.const_int(ch as u64, false))
          .collect::<Vec<_>>();

        let size = text.len() as u32;

        let global = self.module.add_global(
          i8_type.array_type(size)
            .as_basic_type_enum(),
          Some(AddressSpace::default()),
          "byte_text"
        );

        let value = i8_type.const_array(values.as_slice());
        global.set_initializer(&value);

        global.as_basic_value_enum()
      },
      Literal::CString(text) => {
        let global = unsafe {
          self.builder.build_global_string(text, "c_text")
        };

        global.as_basic_value_enum()
      },
      Literal::Char(_) => todo!("generate_literal char"),
      Literal::ByteChar(_) => todo!("generate_literal bytechar"),
      Literal::FloatLiteral(_) => todo!("generate_literal floatliteral"),
      Literal::IntLiteral(text) => {
        let value = parse_int_literal(text);

        let generated_ty = self.generate_type(ty)?
          .to_basic_metadata();

        if is_pointer(ty) {
          if value != 0 {
            panic!("nonzero pointer initialization from literal");
          };

          generated_ty
            .into_pointer_type()
            .const_null()
            .as_basic_value_enum()
        } else {
          // what is "sign_extend" here? (the magic `false` at the end of the next line)
          generated_ty
            .into_int_type()
            .const_int(value, false)
            .as_basic_value_enum()
        }
      },
    })
  }
}
