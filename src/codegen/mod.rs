/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

mod errors;

use errors::*;
use inkwell::types::{FunctionType, VoidType};
use inkwell::values::FunctionValue;

use crate::aster::ast::NamespaceAST;

// use inkwell::context::Context;
// use inkwell::builder::Builder;
// use inkwell::module::Module;

use inkwell::context::Context;
// use inkwell::values::{FloatValue, FunctionValue};
// use inkwell::FloatPredicate;
use inkwell::{
  builder::Builder,
  // values::BasicValueEnum,
  types::{
    IntType,
    BasicMetadataTypeEnum,
  },
  module::Module,
  // values::PointerValue
};

pub struct Codegen<'a, 'ctx> {
  pub context: &'ctx Context,
  pub module: &'a Module<'ctx>,
  pub builder: &'a Builder<'ctx>,
}

impl Codegen<'_, '_> {
  // pub fn init(&mut self, filename: &str) {
  //   self.module.set_source_file_name(filename);
  //   self.generate_main_fn();
  //   self.add_printf();
  // }
}

#[derive(Debug)]
enum MetadataType<'ctx> {
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
      MetadataType::Void(r#void) => unimplemented!("generate basic metadata type (for arg type): void") /* BasicMetadataTypeEnum::VoidType(*r#void) */,
      MetadataType::Enum(BasicMetadataTypeEnum::IntType(r#int)) => BasicMetadataTypeEnum::IntType(*r#int),
      _ => todo!("to_basic_metadata {self:#?}")
    }
  }
}

use crate::aster::{
  ast::*,
  intrinsics::Intrinsic
};

impl<'a, 'ctx> Codegen<'a, 'ctx> {
  fn generate_intrinsic_type(&self, intrinsic: &Intrinsic) -> CodeGenResult<MetadataType<'ctx>> {
    match intrinsic {
      Intrinsic::VOID => Ok(MetadataType::Void(self.context.void_type())),
      Intrinsic::BOOL => Ok(MetadataType::Enum(BasicMetadataTypeEnum::IntType(self.context.bool_type()))),
      Intrinsic::CHAR => Ok(MetadataType::Enum(BasicMetadataTypeEnum::IntType(self.context.i8_type()))),
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

  fn generate_arg_type(&self, ty: &Type) -> CodeGenResult<MetadataType<'ctx>> {
    match ty {
      Type::Intrinsic(intrinsic) => self.generate_intrinsic_type(intrinsic),
      Type::Function(_) => todo!("generate_arg_type function"),
      Type::MemberFunction(_) => todo!("generate_arg_type memberfunction"),
      Type::Struct(_) => todo!("generate_arg_type struct"),
      Type::ConstReferenceTo(_) => todo!("generate_arg_type constreferenceto"),
      Type::MutReferenceTo(_) => todo!("generate_arg_type mutreferenceto"),
      Type::ConstPtrTo(_) => todo!("generate_arg_type constptrto"),
      Type::MutPtrTo(_) => todo!("generate_arg_type mutptrto"),
      Type::ArrayOf(_, _) => todo!("generate_arg_type arrayof"),
      Type::Defined(_) => todo!("generate_arg_type defined"),
      Type::Unknown(_) => todo!("generate_arg_type unknown"),
      Type::UnresolvedNumeric(_) => todo!("generate_arg_type unresolvednumeric"),
      Type::Unresolved => todo!("generate_arg_type unresolved"),
    }
  }

  fn declare_function(&mut self, func: &FunctionAST) -> CodeGenResult<FunctionValue<'ctx>> {
    let decl = &func.decl;

    let ret_ty = self.generate_arg_type(&decl.ret.e)?;
    let args = decl
      .args
      .values()
      .map(
        |ast|
          self.generate_arg_type(&ast.e)
      )
      .collect::<Result<Vec<_>, _>>()?;

    let args = args
      .iter()
      .map(|ty| ty.to_basic_metadata())
      .collect::<Vec<_>>();

    let func_ty = ret_ty.fn_type(args.as_slice(), false);

    let name = &func.decl.ident.text;
    Ok(self.module.add_function(name, func_ty, None))
  }

  fn generate_function(&mut self, ast: &FunctionAST, value: FunctionValue<'ctx>) -> CodeGenResult<()> {
    let block = self.context.append_basic_block(value, "entry");
    self.builder.position_at_end(block);

    for expr in ast.body.children.iter() {
      todo!("generate block expression");
    };

    if ast.body.returns_last {
      todo!("returns last");
    } else {
      self.builder.build_return(None);
    };

    Ok(())
  }

  fn generate_namespace(&mut self, ns: &NamespaceAST) -> CodeGenResult<()> {
    let mut asts_values: Vec<(&FunctionAST, FunctionValue<'ctx>)> = vec![];

    for (name, structure) in ns.map.iter() {
      match structure {
        Structure::Namespace(ns) => {
          self.generate_namespace(ns)?;
        },
        Structure::Function(func) => {
          asts_values.push((
            func,
            self.declare_function(func)?
          ));
        },
        _ => {}
      };
    };

    for (ast, value) in asts_values {
      self.generate_function(ast, value)?;
    };

    Ok(())
  }

  pub fn generate(&mut self, global: &NamespaceAST) -> CodeGenResult<()> {
    self.generate_namespace(global)?;

    if let Err(err) = self.module.verify() {
      return ValidationFailedSnafu {
        message: err.to_string()
      }.fail();
    };

    Ok(())
  }
}
