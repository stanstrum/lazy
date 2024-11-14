use inkwell::{
  builder::Builder,
  values::{BasicValue, BasicValueEnum},
};

use super::*;

impl lang::LiteralInstruction {
  fn generate_with_builder<'ctx, W: CompilerWorkflow>(
    &self,
    generator: &Generator<W>,
    _builder: &Builder,
  ) -> Result<BasicValueEnum<'ctx>> {
    match &*self.kind {
      lang::LiteralInstructionKind::Integer(value) => Ok(
        self
          .ty
          .g_type_of(ContextOrRef::Context(&generator.context))?
          .as_basic_type_enum()
          .into_int_type()
          .const_int(*value, false)
          .as_basic_value_enum(),
      ),
      lang::LiteralInstructionKind::Float(_) => {
        todo!()
      },
      lang::LiteralInstructionKind::String(_) => {
        todo!()
      },
    }
  }
}

impl lang::Instruction {
  fn generate_with_builder<'ctx, W: CompilerWorkflow>(
    this: &RcCell<Self>,
    generator: &Generator<W>,
    builder: &Builder,
  ) -> Result<Option<BasicValueEnum<'ctx>>> {
    match &*this.borrow() {
      lang::Instruction::Literal(literal) => {
        Ok(Some(literal.generate_with_builder(generator, builder)?))
      },
      lang::Instruction::Block(this) => {
        let mut scope = vec![];
        for variable in this.borrow().variables.iter() {
          let variable = variable.borrow();

          // let name = variable.name.name.to_owned();
          let context = (&generator.context).into();
          let ty = variable.ty.g_type_of(context)?;

          builder.build_alloca(ty.as_basic_type_enum(), "alloca");

          scope.push(ty);
        }

        for instruction in this.borrow().instructions.iter() {
          lang::Instruction::generate_with_builder(instruction, generator, builder)?;
        }

        Ok(None)
      },
      lang::Instruction::Return { value , .. } => {
        let value = {
          if let Some(value) = value {
            lang::Instruction::generate_with_builder(value, generator, builder)?
          } else {
            None
          }
        };

        builder.build_return(value.as_ref().map(|x| x as _));

        Ok(None)
      },
      lang::Instruction::ImplicitReturnLast { .. } => todo!(),
    }
  }
}

impl lang::FunctionBlock {
  fn generate_in_function<W: CompilerWorkflow>(
    this: &RcCell<Self>,
    generator: &mut Generator<W>,
    function: &FunctionValue,
  ) -> Result {
    let builder = generator.context.create_builder();
    let basic_block = generator.context.append_basic_block(*function, "entry");
    builder.position_at_end(basic_block);

    warn!("{}: generate_in_function", crate::enchant!("stub"));

    let mut scope = vec![];
    for variable in this.borrow().variables.iter() {
      let variable = variable.borrow();

      // let name = variable.name.name.to_owned();
      let context = (&generator.context).into();
      let ty = variable.ty.g_type_of(context)?;

      scope.push(ty.as_basic_metadata_type());

      todo!();
    }

    for instruction in this.borrow().children.iter() {
      lang::Instruction::generate_with_builder(instruction, generator, &builder)?;
    }

    ok
  }
}

impl lang::Function {
  pub(super) fn generate_in_module<W: CompilerWorkflow>(
    this: &RcCell<Self>,
    generator: &mut Generator<W>,
    module: &inkwell::module::Module,
  ) -> Result {
    let context = module.get_context();

    let mut should_push_value = false;
    let function = if this.borrow().generator_id.is_some() {
      let id = this.borrow().generator_id.unwrap();

      generator.functions[id]
    } else {
      should_push_value = true;

      let context = ContextOrRef::Ref(&context);
      let function_ty = this.g_type_of(context)?;
      let function = module.add_function(&this.borrow().name.name, function_ty, None);

      let id = generator.functions.len();

      this.borrow_mut().generator_id = Some(id);

      function
    };

    let result =
      lang::FunctionBlock::generate_in_function(&this.borrow().body, generator, &function);

    // don't drop our value
    if should_push_value {
      generator.functions.push(unsafe {
        #[allow(clippy::missing_transmute_annotations)]
        std::mem::transmute(function)
      });
    };

    result
  }
}

impl TypeOf for lang::Function {
  type Out<'ctx> = FunctionType<'ctx>;

  fn g_type_of<'a, 'ctx: 'a>(&self, context: ContextOrRef<'a, 'ctx>) -> Result<Self::Out<'ctx>> {
    let param_types: Vec<_> = self
      .arguments
      .iter()
      .map(|argument| {
        Ok(
          argument
            .borrow()
            .ty
            .g_type_of(context)?
            .as_basic_metadata_type(),
        )
      })
      .collect::<Result<_>>()?;

    Ok(
      self
        .return_ty
        .g_type_of(context)?
        .fn_type(&param_types, false),
    )
  }
}
