/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use inkwell::{types::BasicMetadataTypeEnum, values::BasicValueEnum};

use crate::aster::ast::{BindingAST, VariableReference};

use super::{
  Codegen,
  CodeGenResult
};

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

      self.builder.build_store(ptr, value);
    };

    self.var_map.insert(VariableReference::ResolvedVariable(ast), BasicValueEnum::PointerValue(ptr));

    Ok(())
  }
}
