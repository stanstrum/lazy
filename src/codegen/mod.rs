/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

mod errors;

use std::collections::HashMap;

use errors::*;
use inkwell::AddressSpace;
use inkwell::types::{FunctionType, VoidType, PointerType, ArrayType, BasicType};
use inkwell::values::{FunctionValue, BasicValueEnum, BasicValue, PointerValue};

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
    // IntType,
    BasicMetadataTypeEnum,
  },
  module::Module,
  // values::PointerValue
};

use crate::aster::consts;

pub struct Codegen<'a, 'ctx> {
  pub context: &'ctx Context,
  pub module: &'a Module<'ctx>,
  pub builder: &'a Builder<'ctx>,

  pub var_map: HashMap<VariableReference, BasicValueEnum<'ctx>>
}

fn parse_int_literal(text: &String) -> u64 {
  if text.starts_with(consts::punctuation::BIN_PFX) {
    todo!("parse_int_literal bin");
  };

  if text.starts_with(consts::punctuation::OCT_PFX) {
    todo!("parse_int_literal oct");
  };

  if text.starts_with(consts::punctuation::HEX_PFX) {
    todo!("parse_int_literal hex");
  };

  let clean = text.chars().filter(|ch| *ch != '_').collect::<String>();

  clean.parse()
    .expect("failed to parse int literal")
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
      Type::Unknown(_) => todo!("error: generate type unknown"),
      Type::UnresolvedLiteral(_) => todo!("error: generate type unresolved literal"),
      Type::Unresolved => todo!("error: generate type unresolved"),
    }
  }

  fn declare_function(&mut self, func: &FunctionAST) -> CodeGenResult<FunctionValue<'ctx>> {
    let decl = &func.decl;

    let ret_ty = self.generate_type(&decl.ret.e)?;

    let mut args = decl
      .args
      .values()
      .collect::<Vec<_>>();

    args.sort_by_key(|ty_ast| ty_ast.span().start);

    let args = args.iter().map(
        |ast| self.generate_type(&ast.e)
      ).collect::<Result<Vec<_>, _>>()?;

    let args = args
      .iter()
      .map(|ty| ty.to_basic_metadata())
      .collect::<Vec<_>>();

    let func_ty = ret_ty.fn_type(args.as_slice(), false);

    let name = &func.decl.ident.text;
    Ok(self.module.add_function(name, func_ty, None))
  }

  fn generate_binding(&mut self, ast: &BindingAST) -> CodeGenResult<()> {
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

  fn generate_literal(&mut self, lit: &LiteralAST, ty: &Type) -> CodeGenResult<BasicValueEnum<'ctx>> {
    Ok(match &lit.l {
      Literal::UnicodeString(_) => todo!("generate_literal unicodestring"),
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

  fn generate_atom(&mut self, ast: &AtomExpressionAST) -> CodeGenResult<Option<BasicValueEnum<'ctx>>> {
    Ok(match &ast.a {
      AtomExpression::Literal(lit) => Some(self.generate_literal(lit, &ast.out)?),
      AtomExpression::Variable(qual, var_ref)
        if matches!(var_ref, VariableReference::ResolvedVariable(_)) =>
      {
        let name = qual.parts.last().unwrap().text.as_str();

        let ptr = self.var_map
          .get(var_ref)
          .expect("we don't have this variablereference")
          .into_pointer_value();

        let value = self.builder.build_load(ptr, name);

        Some(value)
      },
      AtomExpression::Variable(qual, var_ref)
        if matches!(var_ref, VariableReference::ResolvedArgument(_)) =>
      {
        let value = self.var_map.get(var_ref)
          .expect("we don't have this variablereference")
          .to_owned();

        Some(value)
      },
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
      let mut ast_params = ast.decl.args
        .values()
        .collect::<Vec<_>>();

      ast_params.sort_by_key(|ty_ast| ty_ast.span().start);

      for (param, ty) in value.get_param_iter().zip(ast_params.iter()) {
        let var_ref = VariableReference::ResolvedArgument(*ty);

        self.var_map.insert(
          var_ref,
          param
            .try_into()
            .expect("failed to convert param to ptr")
        );
      };

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
