use crate::aster::pprint::Pretty;

use super::*;

mod reimpl;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(super) enum LazyType<'ctx> {
  Void(inkwell::types::VoidType<'ctx>),
  Int(inkwell::types::IntType<'ctx>),
  Float(inkwell::types::FloatType<'ctx>),
  Pointer(inkwell::types::PointerType<'ctx>),
  Struct(inkwell::types::StructType<'ctx>),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(super) enum LazyValue<'ctx> {
  Void,
  Int(inkwell::values::IntValue<'ctx>),
  Pointer(inkwell::values::PointerValue<'ctx>),
  Struct(inkwell::values::StructValue<'ctx>),
}

fn make_intrinsic_type<'ctx>(comp: &Compilation<'_, '_, 'ctx>, intrinsic: ::token::intrinsic::Intrinsic) -> LazyType<'ctx> {
  match intrinsic {
    ::token::intrinsic::Intrinsic::Void => comp.llvm.context.void_type().into(),
    ::token::intrinsic::Intrinsic::Bool => comp.llvm.context.bool_type().into(),
    ::token::intrinsic::Intrinsic::I8  | ::token::intrinsic::Intrinsic::U8  => comp.llvm.context.i8_type().into(),
    ::token::intrinsic::Intrinsic::I16 | ::token::intrinsic::Intrinsic::U16 => comp.llvm.context.i16_type().into(),
    ::token::intrinsic::Intrinsic::I32 | ::token::intrinsic::Intrinsic::U32 => comp.llvm.context.i32_type().into(),
    ::token::intrinsic::Intrinsic::I64 | ::token::intrinsic::Intrinsic::U64 => comp.llvm.context.i64_type().into(),
    ::token::intrinsic::Intrinsic::F32 => comp.llvm.context.f32_type().into(),
    ::token::intrinsic::Intrinsic::F64 => comp.llvm.context.f64_type().into(),
  }
}

fn make_param_types<'ctx>(
  comp: &mut Compilation<'_, '_, 'ctx>,
  function: lang::reference::FunctionReference,
) -> Result<Vec<inkwell::types::BasicMetadataTypeEnum<'ctx>>> {
  let borrow = comp.lazy.rget(function);

  borrow.header.arguments.iter()
    .map(|var| {
      let ty = make_type(comp, &var.ty)?;

      Ok(inkwell::types::BasicMetadataTypeEnum::from(ty))
    })
    .collect()
}

pub(super) fn make_function_type<'ctx>(
  comp: &mut Compilation<'_, '_, 'ctx>,
  function: lang::reference::FunctionReference,
) -> Result<inkwell::types::FunctionType<'ctx>> {
  let borrow = comp.lazy.rget(function);

  let param_types = make_param_types(comp, function)?;

  // SPONGE: need to determine this from AST
  let is_var_args = false;

  let ret_ty = make_type(comp, &borrow.header.ret_ty)?;
  let function_type = ret_ty.fn_type(&param_types, is_var_args);

  Ok(function_type)
}

pub(super) fn make_type<'ctx>(comp: &Compilation<'_, '_, 'ctx>, t: &impl TypeOf) -> Result<LazyType<'ctx>> {
  let ty = t.type_of(comp.lazy)
    .expect("type of to be Some()");

  match ty {
    lang::ty::Type::Reference(_) => todo!(),
    lang::ty::Type::Resolved { .. } => todo!(),
    lang::ty::Type::Intrinsic { kind, .. } => Ok(make_intrinsic_type(comp, kind)),
    lang::ty::Type::ReferenceTo { .. } => {
      Ok(LazyType::Pointer(
        comp.llvm.context.ptr_type(Default::default())
      ))
    },
    lang::ty::Type::UnsizedArrayOf { .. } => todo!(),
    lang::ty::Type::SizedArrayOf { .. } => todo!(),

    lang::ty::Type::Unresolved { .. } => todo!(),
    | lang::ty::Type::WeakInteger { span, .. }
    | lang::ty::Type::WeakFloat { span, .. }
    | lang::ty::Type::WeakString { span, .. }
    | lang::ty::Type::Weak { span, .. } => {
      Err(Error::StillUnresolved {
        what: "type".into(),
        note: format!("is {}", ty.print(comp.lazy)),
        span,
      })
    },
    lang::ty::Type::Struct { prototype } => {
      let field_types = comp.lazy.rget(prototype).members.iter()
        .map(|variable| {
          make_type(comp, &variable.ty)
            .map(LazyType::as_basic_type_enum)
            .map(Option::unwrap)
        })
        .collect::<Result<Vec<_>>>()?;

      // SPONGE: packed is not implemented
      let struct_type = comp.llvm.context.struct_type(&field_types, false);

      Ok(LazyType::Struct(struct_type))
    },
  }
}
