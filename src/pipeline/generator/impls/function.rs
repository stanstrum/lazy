use super::*;

impl lang::FunctionBlock {
  fn generate_in_function<W: CompilerWorkflow>(_this: &RcCell<Self>, generator: &mut Generator<W>, function: &FunctionValue) -> Result {
    let builder = generator.context.create_builder();
    let basic_block = generator.context.append_basic_block(*function, "entry");
    builder.position_at_end(basic_block);

    warn!("FunctionBlock stub");

    if function.get_type().get_return_type().is_none() {
      builder.build_return(None);
    };

    ok
  }
}

impl lang::Function {
  pub(super) fn generate_in_module<W: CompilerWorkflow>(this: &RcCell<Self>, generator: &mut Generator<W>, module: &inkwell::module::Module) -> Result {
    let context = module.get_context();

    let function = if this.borrow().generator_id.is_some() {
      let id = this.borrow().generator_id.unwrap();

      generator.functions[id]
    } else {
      let function_ty = this.g_type_of(&context)?;
      let function = module.add_function(&this.borrow().name.name, function_ty, None);

      trace!("{function:?}");

      let id = generator.functions.len();

      generator.functions.push(unsafe {
        #[allow(clippy::missing_transmute_annotations)]
        std::mem::transmute(function)
      });

      this.borrow_mut().generator_id = Some(id);

      module.add_function(
        &this.borrow().name.name,
        this.g_type_of(&context)?,
        None
      );

      function
    };

    lang::FunctionBlock::generate_in_function(&this.borrow().body, generator, &function)
  }
}

impl TypeOf for lang::Function {
  type Out<'ctx> = FunctionType<'ctx>;

  fn g_type_of<'ctx>(&self, context: &ContextRef<'ctx>) -> Result<Self::Out<'ctx>> {
    let param_types: Vec<_> = self.arguments.iter()
      .map(|argument| {
        Ok(argument.borrow().ty.g_type_of(context)?.as_basic_metadata_type())
      })
      .collect::<Result<_>>()?;

    Ok(self.return_ty.g_type_of(context)?.fn_type(&param_types, false))
  }
}
