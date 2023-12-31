/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use inkwell::{
  types::{
    BasicMetadataTypeEnum,
    BasicTypeEnum
  },
  AddressSpace
};

use crate::aster::{
  intrinsics::Intrinsic,
  ast::*
};

// use crate::codegen::parse_int_literal;

use super::{
  Codegen,
  CodeGenResult,
  metadatatype::MetadataType,
  errors::*
};

impl<'a, 'ctx> Codegen<'a, 'ctx> {
  pub fn generate_intrinsic_type(&self, intrinsic: &Intrinsic) -> CodeGenResult<MetadataType<'ctx>> {
    match intrinsic {
      Intrinsic::VOID => Ok(MetadataType::Void(self.context.void_type())),
      Intrinsic::BOOL => Ok(MetadataType::Enum(BasicMetadataTypeEnum::IntType(self.context.bool_type()))),
      Intrinsic::CHAR => Ok(MetadataType::Enum(BasicMetadataTypeEnum::IntType(self.context.i32_type()))),
      Intrinsic::U8 => Ok(MetadataType::Enum(BasicMetadataTypeEnum::IntType(self.context.i8_type()))),
      Intrinsic::U16 => Ok(MetadataType::Enum(BasicMetadataTypeEnum::IntType(self.context.i16_type()))),
      Intrinsic::U32 => Ok(MetadataType::Enum(BasicMetadataTypeEnum::IntType(self.context.i32_type()))),
      Intrinsic::U64 => Ok(MetadataType::Enum(BasicMetadataTypeEnum::IntType(self.context.i64_type()))),
      Intrinsic::USIZE => Ok(MetadataType::Enum(BasicMetadataTypeEnum::IntType(self.context.i64_type()))),
      Intrinsic::I8 => Ok(MetadataType::Enum(BasicMetadataTypeEnum::IntType(self.context.i8_type()))),
      Intrinsic::I16 => Ok(MetadataType::Enum(BasicMetadataTypeEnum::IntType(self.context.i16_type()))),
      Intrinsic::I32 => Ok(MetadataType::Enum(BasicMetadataTypeEnum::IntType(self.context.i32_type()))),
      Intrinsic::I64 => Ok(MetadataType::Enum(BasicMetadataTypeEnum::IntType(self.context.i64_type()))),
      Intrinsic::ISIZE => Ok(MetadataType::Enum(BasicMetadataTypeEnum::IntType(self.context.i64_type()))),
    }
  }

  pub fn generate_type(&mut self, ty: &Type) -> CodeGenResult<MetadataType<'ctx>> {
    match ty {
      Type::Intrinsic(intrinsic) => self.generate_intrinsic_type(intrinsic),
      Type::Function(_) => todo!("generate_type function"),
      Type::External(_) => todo!("generate_type external"),
      Type::Struct(_, members) => {
        let mut field_types: Vec<BasicTypeEnum> = Vec::with_capacity(members.len());

        for (ty, _) in members.iter() {
          let ty = self.generate_type(ty)?
            .to_basic_metadata();

          field_types.push(ty.try_into().unwrap());
        };

        let ty = self.context.struct_type(field_types.as_slice(), false);

        Ok(MetadataType::Enum(BasicMetadataTypeEnum::StructType(ty)))
      },
      Type::ConstReferenceTo(referenced) => {
        let ir_ty = self.generate_type(&referenced.e)?;

        Ok(MetadataType::Enum(
          BasicMetadataTypeEnum::PointerType(
            ir_ty.ptr_ty(AddressSpace::default())
          )
        ))
      },
      Type::MutReferenceTo(_) => todo!("generate_arg_type mutreferenceto"),
      Type::ConstPtrTo(_) => todo!("generate_arg_type constptrto"),
      Type::MutPtrTo(_) => todo!("generate_arg_type mutptrto"),
      Type::ArrayOf(_count, item) => {
        let ir_ty = self.generate_type(&item.e)?;

        // use crate::aster::

        // if count.is_some() {
        //   // Ok(MetadataType::Enum(
        //   //   ir_ty.array_type(0);
        //   //   BasicMetadataTypeEnum::ArrayType(
        //   //     ir_ty
        //   //   )
        //   // ))

        //   let item_ty = self.generate_type(&item.e)?;

        //   let lit = &count.as_ref().unwrap().l;
        //   let Literal::IntLiteral(text) = lit else { unreachable!(); };

        //   Ok(MetadataType::Enum(
        //     BasicMetadataTypeEnum::ArrayType(
        //       item_ty.array_type(parse_int_literal(text) as u32)
        //     )
        //   ))
        // } else {
        //   // c undefined-length arrays just exploit pointer math ...
        //   // practically, there is no difference in type information
        //   // between a pointer to an int and a pointer to an int
        //   // followed by more ints

        //   Ok(ir_ty)
        // }

        Ok(ir_ty)
      },
      Type::Defined(ast) => {
        let ast = unsafe { &**ast };

        self.generate_type(&ast.e)
      },
      Type::Generic(ident, ..) => {
        return AtSpanSnafu {
          msg: "Cannot generate LLVM for a generic type",
          span: ident.span(),
        }.fail();
      },
      Type::Unknown(_) => todo!("error: generate type unknown"),
      Type::UnresolvedLiteral(_) => todo!("error: generate type unresolved literal"),
      Type::Unresolved => todo!("error: generate type unresolved"),
    }
  }
}
