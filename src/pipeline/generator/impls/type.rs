use super::*;

impl TypeOf for lang::Intrinsic {
  type Out<'ctx> = GeneratorSuperType<'ctx>;

  fn g_type_of<'ctx>(&self, context: &ContextRef<'ctx>) -> Result<Self::Out<'ctx>> {
    Ok(match self {
      lang::Intrinsic::Void => GeneratorSuperType::Void(context.void_type()),
      lang::Intrinsic::U8 => todo!(),
      lang::Intrinsic::U16 => todo!(),
      lang::Intrinsic::U32 => todo!(),
      lang::Intrinsic::U64 => todo!(),
      lang::Intrinsic::I8 => todo!(),
      lang::Intrinsic::I16 => todo!(),
      lang::Intrinsic::I32 => todo!(),
      lang::Intrinsic::I64 => todo!(),
      lang::Intrinsic::F16 => todo!(),
      lang::Intrinsic::F32 => todo!(),
      lang::Intrinsic::F64 => todo!(),
    })
  }
}

impl TypeOf for lang::Type<Module> {
  type Out<'ctx> = GeneratorSuperType<'ctx>;

  fn g_type_of<'ctx>(&self, context: &ContextRef<'ctx>) -> Result<Self::Out<'ctx>> {
    match self {
      lang::Type::Intrinsic { kind, .. } => kind.g_type_of(context),
      lang::Type::Reference(_) => todo!(),
    }
  }
}
