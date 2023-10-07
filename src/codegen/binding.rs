/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use inkwell::{
  types::{
    BasicMetadataTypeEnum,
    AnyTypeEnum,
    BasicTypeEnum
  },
  values::BasicValueEnum
};

use crate::aster::ast::{
  BindingAST,
  VariableReference
};

use super::{
  Codegen,
  CodeGenResult
};

fn to_basic_type<'ctx>(any: AnyTypeEnum<'ctx>) -> BasicTypeEnum<'ctx> {
  match any {
    AnyTypeEnum::ArrayType(ty) => BasicTypeEnum::ArrayType(ty),
    AnyTypeEnum::FloatType(ty) => BasicTypeEnum::FloatType(ty),
    AnyTypeEnum::IntType(ty) => BasicTypeEnum::IntType(ty),
    AnyTypeEnum::PointerType(ty) => BasicTypeEnum::PointerType(ty),
    AnyTypeEnum::StructType(ty) => BasicTypeEnum::StructType(ty),
    AnyTypeEnum::VectorType(ty) => BasicTypeEnum::VectorType(ty),
    _ => panic!("invalid type coerced to basic type")
  }
}

impl<'a, 'ctx> Codegen<'a, 'ctx> {
  pub fn generate_binding(&mut self, ast: &BindingAST) -> CodeGenResult<()> {
    let name = ast.ident.text.as_str();

    let ty = self.generate_type(
      &ast.ty
        .as_ref()
        .expect("unresolved type in binding")
        .e
    )?;

    // todo: find a way to fix this issue
    // seems to be a widespread problem in rust
    let ptr = {
      match ty.to_basic_metadata() {
        BasicMetadataTypeEnum::ArrayType(ty) => self.builder.build_alloca(ty, name),
        BasicMetadataTypeEnum::FloatType(ty) => self.builder.build_alloca(ty, name),
        BasicMetadataTypeEnum::IntType(ty) => self.builder.build_alloca(ty, name),
        BasicMetadataTypeEnum::PointerType(ty) => self.builder.build_alloca(ty, name),
        BasicMetadataTypeEnum::StructType(ty) => self.builder.build_alloca(ty, name),
        BasicMetadataTypeEnum::VectorType(ty) => self.builder.build_alloca(ty, name),
        BasicMetadataTypeEnum::MetadataType(_) => {
          unreachable!("metadata type as binding type");
        },
      }
    };

    if ast.value.is_some() {
      let value = self.generate_expr(
        ast.value.as_ref().unwrap()
      )?.expect("value expr did not return a value");

      let casted_value = self.builder.build_bitcast(
          value,
          to_basic_type(
            ptr.get_type().get_element_type()
          ),
          "cast"
        );

      self.builder.build_store(ptr, casted_value);
    };

    self.var_map.insert(VariableReference::ResolvedVariable(ast), BasicValueEnum::PointerValue(ptr));

    Ok(())
  }
}
