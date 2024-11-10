use super::*;

impl lang::FunctionBlock {
  fn generate_in_function<'ctx, W: CompilerWorkflow>(this: &RcCell<Self>, generator: &mut Generator<W>, function: &FunctionValue) -> Result {
    let builder = generator.context.create_builder();
    let basic_block = generator.context.append_basic_block(*function, "entry");
    builder.position_at_end(basic_block);

    warn!("{}: generate_in_function", crate::enchant!("stub"));

    let mut scope = vec![];
    for variable in this.borrow().variables.iter() {
      let variable = variable.borrow();

      // let name = variable.name.name.to_owned();
      let context = (&generator.context).into();
      let ty = variable.ty.g_type_of(context)
        ?.as_basic_metadata_type();

      scope.push(ty);

      todo!();
    };

    if function.get_type().get_return_type().is_none() {
      builder.build_return(None);
    };

    ok
  }
}

impl lang::Function {
  pub(super) fn generate_in_module<'a, 'ctx, W: CompilerWorkflow>(this: &RcCell<Self>, generator: &mut Generator<W>, module: &'a inkwell::module::Module<'ctx>) -> Result {
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

    let result = lang::FunctionBlock::generate_in_function(&this.borrow().body, generator, &function);

    // don't drop our value
    if should_push_value {
      generator.functions.push(unsafe {
        std::mem::transmute(function)
      });
    };

    result
  }
}

impl TypeOf for lang::Function {
  type Out<'ctx> = FunctionType<'ctx>;

  fn g_type_of<'a, 'ctx: 'a>(&self, context: ContextOrRef<'a, 'ctx>) -> Result<Self::Out<'ctx>> {
    let param_types: Vec<_> = self.arguments.iter()
      .map(|argument| {
        Ok(argument.borrow().ty.g_type_of(context)?.as_basic_metadata_type())
      })
      .collect::<Result<_>>()?;

    Ok(self.return_ty.g_type_of(context)?.fn_type(&param_types, false))
  }
}
