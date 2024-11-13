use inkwell::types::BasicType;

use super::*;
use crate::translator::ReferenceResolve;

impl TypeOf for lang::Intrinsic {
  type Out<'ctx> = GeneratorSuperType<'ctx>;

  fn g_type_of<'a, 'ctx: 'a>(&self, context: ContextOrRef<'a, 'ctx>) -> Result<Self::Out<'ctx>> {
    // SPONGE: deal with this later
    #[allow(clippy::missing_transmute_annotations)]
    Ok(unsafe {
      std::mem::transmute(match self {
        lang::Intrinsic::Void => GeneratorSuperType::Void(context.void_type()),
        lang::Intrinsic::U8 => GeneratorSuperType::Basic(
          match &context {
            ContextOrRef::Context(ctx) => ctx.i8_type(),
            ContextOrRef::Ref(ctx) => ctx.i8_type(),
          }
          .as_basic_type_enum(),
        ),
        lang::Intrinsic::U16 => GeneratorSuperType::Basic(
          match &context {
            ContextOrRef::Context(ctx) => ctx.i16_type(),
            ContextOrRef::Ref(ctx) => ctx.i16_type(),
          }
          .as_basic_type_enum(),
        ),
        lang::Intrinsic::U32 => GeneratorSuperType::Basic(
          match &context {
            ContextOrRef::Context(ctx) => ctx.i32_type(),
            ContextOrRef::Ref(ctx) => ctx.i32_type(),
          }
          .as_basic_type_enum(),
        ),
        lang::Intrinsic::U64 => GeneratorSuperType::Basic(
          match &context {
            ContextOrRef::Context(ctx) => ctx.i64_type(),
            ContextOrRef::Ref(ctx) => ctx.i64_type(),
          }
          .as_basic_type_enum(),
        ),
        lang::Intrinsic::I8 => GeneratorSuperType::Basic(
          match &context {
            ContextOrRef::Context(ctx) => ctx.i8_type(),
            ContextOrRef::Ref(ctx) => ctx.i8_type(),
          }
          .as_basic_type_enum(),
        ),
        lang::Intrinsic::I16 => GeneratorSuperType::Basic(
          match &context {
            ContextOrRef::Context(ctx) => ctx.i16_type(),
            ContextOrRef::Ref(ctx) => ctx.i16_type(),
          }
          .as_basic_type_enum(),
        ),
        lang::Intrinsic::I32 => GeneratorSuperType::Basic(
          match &context {
            ContextOrRef::Context(ctx) => ctx.i32_type(),
            ContextOrRef::Ref(ctx) => ctx.i32_type(),
          }
          .as_basic_type_enum(),
        ),
        lang::Intrinsic::I64 => GeneratorSuperType::Basic(
          match &context {
            ContextOrRef::Context(ctx) => ctx.i64_type(),
            ContextOrRef::Ref(ctx) => ctx.i64_type(),
          }
          .as_basic_type_enum(),
        ),
        lang::Intrinsic::F16 => GeneratorSuperType::Basic(
          match &context {
            ContextOrRef::Context(ctx) => ctx.f16_type(),
            ContextOrRef::Ref(ctx) => ctx.f16_type(),
          }
          .as_basic_type_enum(),
        ),
        lang::Intrinsic::F32 => GeneratorSuperType::Basic(
          match &context {
            ContextOrRef::Context(ctx) => ctx.f32_type(),
            ContextOrRef::Ref(ctx) => ctx.f32_type(),
          }
          .as_basic_type_enum(),
        ),
        lang::Intrinsic::F64 => GeneratorSuperType::Basic(
          match &context {
            ContextOrRef::Context(ctx) => ctx.f64_type(),
            ContextOrRef::Ref(ctx) => ctx.f64_type(),
          }
          .as_basic_type_enum(),
        ),
      })
    })
  }
}

impl TypeOf for lang::Type<Module> {
  type Out<'ctx> = GeneratorSuperType<'ctx>;

  fn g_type_of<'a, 'ctx: 'a>(&self, context: ContextOrRef<'a, 'ctx>) -> Result<Self::Out<'ctx>> {
    match self {
      lang::Type::Intrinsic { kind, .. } => kind.g_type_of(context),
      lang::Type::Reference(reference) => reference
        .get()?
        .unwrap()
        .upgrade()
        .unwrap()
        .g_type_of(context),
      other => todo!("{other:#?}"),
    }
  }
}
