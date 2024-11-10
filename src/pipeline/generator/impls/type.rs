use super::*;

impl TypeOf for lang::Intrinsic {
  type Out<'ctx> = GeneratorSuperType<'ctx>;

  fn g_type_of<'a, 'ctx: 'a>(&self, context: &'a ContextOrRef<'a, 'ctx>) -> Result<Self::Out<'ctx>> {
    Ok(match self {
      lang::Intrinsic::Void => GeneratorSuperType::Void(unsafe { std::mem::transmute(context.void_type()) }),
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

  fn g_type_of<'a, 'ctx: 'a>(&self, context: &'a ContextOrRef<'a, 'ctx>) -> Result<Self::Out<'ctx>> {
    match self {
      lang::Type::Intrinsic { kind, .. } => kind.g_type_of(context),
      lang::Type::Reference(_) => todo!(),
    }
  }
}
