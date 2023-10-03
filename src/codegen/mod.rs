/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

mod errors;

use errors::*;
use inkwell::AddressSpace;
use inkwell::types::{FunctionType, VoidType, PointerType, ArrayType};
use inkwell::values::{FunctionValue, BasicValueEnum, BasicValue};

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
      MetadataType::Enum(basic_metadata_enum) => *basic_metadata_enum,
    }
  }

  pub fn ptr_ty(&self, address_space: AddressSpace) -> PointerType<'ctx> {
    match self {
      MetadataType::Void(void) => unimplemented!("ptr to void"),
      MetadataType::Enum(BasicMetadataTypeEnum::IntType(ty)) => ty.ptr_type(address_space),
      MetadataType::Enum(BasicMetadataTypeEnum::PointerType(ty)) => ty.ptr_type(address_space),
      MetadataType::Enum(_) => todo!("ptr_ty {self:#?}"),
    }
  }

  pub fn array_type(&self, size: u32) -> ArrayType<'ctx> {
    match self {
      MetadataType::Void(void) => unimplemented!("array of void"),
      MetadataType::Enum(BasicMetadataTypeEnum::IntType(int)) => int.array_type(size),
      _ => todo!("array_type {self:#?}")
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

  fn generate_type(&self, ty: &Type) -> CodeGenResult<MetadataType<'ctx>> {
    match ty {
      Type::Intrinsic(intrinsic) => self.generate_intrinsic_type(intrinsic),
      Type::Function(_) => todo!("generate_arg_type function"),
      Type::MemberFunction(_) => todo!("generate_arg_type memberfunction"),
      Type::Struct(_) => todo!("generate_arg_type struct"),
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
      Type::ArrayOf(count, item) => {
        let ir_ty = self.generate_type(&item.e)?;

        // use crate::aster::

        if count.is_some() {
          // Ok(MetadataType::Enum(
          //   ir_ty.array_type(0);
          //   BasicMetadataTypeEnum::ArrayType(
          //     ir_ty
          //   )
          // ))

          todo!("sized array");
        } else {
          // c undefined-length arrays just exploit pointer math ...
          // practically, there is no difference in type information
          // between a pointer to an int and a pointer to an int
          // followed by more ints

          Ok(ir_ty)
        }
      },
      Type::Defined(ast) => {
        let ast = unsafe { &**ast };

        self.generate_type(&ast.e)
      },
      Type::Unknown(_) => todo!("generate_arg_type unknown"),
      Type::UnresolvedLiteral(_) => todo!("generate_arg_type unresolvednumeric"),
      Type::Unresolved => todo!("generate_arg_type unresolved"),
    }
  }

  fn declare_function(&mut self, func: &FunctionAST) -> CodeGenResult<FunctionValue<'ctx>> {
    let decl = &func.decl;

    let MetadataType::Enum(ret_ty) = self.generate_type(&decl.ret.e)? else {
      todo!("error: invalid arg type");
    };

    let args = decl
      .args
      .values()
      .map(
        |ast|
          self.generate_type(&ast.e)
      )
      .collect::<Result<Vec<_>, _>>()?;

    let args = args
      .iter()
      .map(|ty| ty.to_basic_metadata())
      .collect::<Vec<_>>();

    let func_ty = MetadataType::Enum(ret_ty).fn_type(args.as_slice(), false);

    let name = &func.decl.ident.text;
    Ok(self.module.add_function(name, func_ty, None))
  }

  fn generate_binding(&mut self, ast: &BindingAST) -> CodeGenResult<()> {
    todo!("generate_binding");
  }

  fn generate_literal(&mut self, lit: &LiteralAST, ty: &Type) -> CodeGenResult<BasicValueEnum<'ctx>> {
    todo!("generate_literal")
  }

  fn generate_atom(&mut self, ast: &AtomExpressionAST) -> CodeGenResult<Option<BasicValueEnum<'ctx>>> {
    Ok(match &ast.a {
      AtomExpression::Literal(lit) => Some(self.generate_literal(lit, &ast.out)?),
      AtomExpression::Variable(_, _) => todo!(),
      AtomExpression::Return(_) => todo!(),
      AtomExpression::Break(_) => todo!(),
    })
  }

  fn generate_expr(&mut self, ast: &Expression) -> CodeGenResult<Option<BasicValueEnum<'ctx>>> {
    match ast {
      Expression::Atom(ast) => self.generate_atom(ast),
      Expression::Block(_) => todo!("generate_expr block"),
      Expression::SubExpression(_) => todo!("generate_expr subexpression"),
      Expression::ControlFlow(_) => todo!("generate_expr controlflow"),
      Expression::BinaryOperator(_) => todo!("generate_expr binaryoperator"),
      Expression::UnaryOperator(_) => todo!("generate_expr unaryoperator"),
    }
  }

  fn generate_block(&mut self, ast: &BlockExpressionAST) -> CodeGenResult<Option<BasicValueEnum<'ctx>>> {
    let Some((last, all_but_last)) = ast.children.split_last() else {
      return Ok(None);
    };

    let BlockExpressionChild::Expression(last) = last else {
      unreachable!("last child of returns_last block expression was a binding");
    };

    for child in all_but_last.iter() {
      match child {
        BlockExpressionChild::Binding(binding) => {
          self.generate_binding(binding)?;
        },
        BlockExpressionChild::Expression(expr) => {
          self.generate_expr(expr)?;
        },
      }
    };

    self.generate_expr(last)
  }

  fn generate_function(&mut self, ast: &FunctionAST, value: FunctionValue<'ctx>) -> CodeGenResult<()> {
    let block = self.context.append_basic_block(value, "entry");
    self.builder.position_at_end(block);

    let returned = self.generate_block(&ast.body)?;

    // this is very strange
    let returned = returned
      .as_ref()
      .map(
        |val| val as &dyn BasicValue<'ctx>
      );

    self.builder.build_return(returned);

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
