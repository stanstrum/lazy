/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use inkwell::{
  types::{
    VoidType, BasicMetadataTypeEnum, FunctionType, PointerType, ArrayType
  },
  AddressSpace
};

#[derive(Debug)]
pub enum MetadataType<'ctx> {
  Void(VoidType<'ctx>),
  Enum(BasicMetadataTypeEnum<'ctx>)
}

impl<'ctx> MetadataType<'ctx> {
  pub fn fn_type(&self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx> {
    match self {
      MetadataType::Void(r#void) => r#void.fn_type(param_types, is_var_args),
      MetadataType::Enum(BasicMetadataTypeEnum::ArrayType(ty)) => ty.fn_type(param_types, is_var_args),
      MetadataType::Enum(BasicMetadataTypeEnum::FloatType(ty)) => ty.fn_type(param_types, is_var_args),
      MetadataType::Enum(BasicMetadataTypeEnum::IntType(ty)) => ty.fn_type(param_types, is_var_args),
      MetadataType::Enum(BasicMetadataTypeEnum::PointerType(ty)) => ty.fn_type(param_types, is_var_args),
      MetadataType::Enum(BasicMetadataTypeEnum::StructType(ty)) => ty.fn_type(param_types, is_var_args),
      MetadataType::Enum(BasicMetadataTypeEnum::VectorType(ty)) => ty.fn_type(param_types, is_var_args),
      MetadataType::Enum(BasicMetadataTypeEnum::MetadataType(ty)) => ty.fn_type(param_types, is_var_args),
    }
  }

  pub fn to_basic_metadata(&self) -> BasicMetadataTypeEnum<'ctx> {
    match self {
      MetadataType::Void(_) => unimplemented!("generate basic metadata type (for arg type): void") /* BasicMetadataTypeEnum::VoidType(*r#void) */,
      MetadataType::Enum(basic_metadata_enum) => *basic_metadata_enum,
    }
  }

  pub fn ptr_ty(&self, address_space: AddressSpace) -> PointerType<'ctx> {
    match self {
      MetadataType::Void(_) => unimplemented!("ptr to void"),
      MetadataType::Enum(BasicMetadataTypeEnum::ArrayType(ty)) => ty.ptr_type(address_space),
      MetadataType::Enum(BasicMetadataTypeEnum::FloatType(ty)) => ty.ptr_type(address_space),
      MetadataType::Enum(BasicMetadataTypeEnum::IntType(ty)) => ty.ptr_type(address_space),
      MetadataType::Enum(BasicMetadataTypeEnum::PointerType(ty)) => ty.ptr_type(address_space),
      MetadataType::Enum(BasicMetadataTypeEnum::StructType(ty)) => ty.ptr_type(address_space),
      MetadataType::Enum(BasicMetadataTypeEnum::VectorType(ty)) => ty.ptr_type(address_space),
      MetadataType::Enum(BasicMetadataTypeEnum::MetadataType(_)) => {
        panic!("metadata type ptr_to?");
      },
    }
  }

  pub fn array_type(&self, size: u32) -> ArrayType<'ctx> {
    match self {
      MetadataType::Void(_) => unimplemented!("array of void"),
      MetadataType::Enum(BasicMetadataTypeEnum::ArrayType(ty)) => ty.array_type(size),
      MetadataType::Enum(BasicMetadataTypeEnum::FloatType(ty)) => ty.array_type(size),
      MetadataType::Enum(BasicMetadataTypeEnum::IntType(ty)) => ty.array_type(size),
      MetadataType::Enum(BasicMetadataTypeEnum::PointerType(ty)) => ty.array_type(size),
      MetadataType::Enum(BasicMetadataTypeEnum::StructType(ty)) => ty.array_type(size),
      MetadataType::Enum(BasicMetadataTypeEnum::VectorType(ty)) => ty.array_type(size),
      _ => panic!("array of metadatatype?")
    }
  }
}
