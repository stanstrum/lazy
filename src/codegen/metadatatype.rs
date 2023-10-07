/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use inkwell::{types::{VoidType, BasicMetadataTypeEnum, FunctionType, PointerType, ArrayType}, AddressSpace};

#[derive(Debug)]
pub enum MetadataType<'ctx> {
  Void(VoidType<'ctx>),
  Enum(BasicMetadataTypeEnum<'ctx>)
}

impl<'ctx> MetadataType<'ctx> {
  pub fn fn_type(&self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx> {
    match self {
      MetadataType::Void(r#void) => r#void.fn_type(param_types, is_var_args),
      MetadataType::Enum(BasicMetadataTypeEnum::IntType(r#int)) => int.fn_type(param_types, is_var_args),
      _ => todo!("fn_type {self:#?}")
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
      MetadataType::Enum(BasicMetadataTypeEnum::IntType(ty)) => ty.ptr_type(address_space),
      MetadataType::Enum(BasicMetadataTypeEnum::PointerType(ty)) => ty.ptr_type(address_space),
      MetadataType::Enum(_) => todo!("ptr_ty {self:#?}"),
    }
  }

  pub fn array_type(&self, size: u32) -> ArrayType<'ctx> {
    match self {
      MetadataType::Void(_) => unimplemented!("array of void"),
      MetadataType::Enum(BasicMetadataTypeEnum::IntType(int)) => int.array_type(size),
      _ => todo!("array_type {self:#?}")
    }
  }
}
