use super::*;

impl<'a> ParseScope<'a> for FunctionArgument {
  type In = ast::FunctionArgument<DefaultWorkflow>;
  type Scope = Function;

  fn parse_scope(translator: &mut Translator<DefaultWorkflow>, compiler: &Compiler<DefaultWorkflow>, input: Self::In, parent: &Option<WeakCell<Self::Scope>>) -> Result<RcCell<Self>> {
    let module = { parent.clone().unwrap().upgrade().unwrap().scope_parent() };

    let ty = translator.parse_scope(compiler, input.ty, &module)?;

    Ok(new_rc_cell(Self {
      name: input.identifier,
      ty,
      parent: parent.as_ref().cloned().unwrap().into(),
    }))
  }
}

impl<'a> ParseScope<'a> for FunctionBlock {
  type In = ast::BlockExpression<DefaultWorkflow>;
  type Scope = Function;

  fn parse_scope(translator: &mut Translator<DefaultWorkflow>, compiler: &Compiler<DefaultWorkflow>, input: Self::In, parent: &Option<WeakCell<Self::Scope>>) -> Result<RcCell<Self>> {
    // instantiate self so our children have parent references
    let this = new_rc_cell(Self {
      parent: Some(parent.clone().unwrap()).into(),
      variables: vec![],
      children: vec![],
    });

    // parent reference for parsing children nodes
    let _child_parent = Some(Rc::downgrade(&this));

    // first, collect all of this scope's variables
    for child in input.children {
      let ast::BlockChild::Binding(binding) = child else {
        // skip the statement if it's not a binding
        continue;
      };

      let ty = match binding.kind {
        ast::BindingKind::OnlyType(ty) => {
          let scope_parent= parent.as_ref().unwrap().upgrade().unwrap().scope_parent();
          translator.parse_scope(compiler, ty, &scope_parent)?
        },
        ast::BindingKind::OnlyExpression(_) => {
          todo!()
        },
        ast::BindingKind::Both { .. } => {
          todo!()
        },
      };

      this.borrow_mut().variables.push(new_rc_cell(Variable { name: binding.identifier, ty }));
    };

    warn!("{}: not parsing function body", enchant!("stub"));

    Ok(this)
  }
}

impl<'a> ParseScope<'a> for Function {
  type In = ast::Function<DefaultWorkflow>;
  type Scope = Module;

  fn parse_scope(translator: &mut Translator<DefaultWorkflow>, compiler: &Compiler<DefaultWorkflow>, input: Self::In, parent: &Option<WeakCell<Self::Scope>>) -> Result<RcCell<Self>> {
    // SPONGE: this is a dummy block that gets destroyed when this scope ends -- this might lead to leaks or duplicates
    let body = new_rc_cell(FunctionBlock {
      parent: None.into(),
      variables: vec![],
      children: vec![],
    });

    let rc = new_rc_cell(Self {
      parent: parent.as_ref().cloned().into(),
      name: input.identifier,
      arguments: vec![],
      body,
      return_ty: new_rc_cell(Type::Intrinsic {
        kind: Intrinsic::Void,
        parent: parent.clone().unwrap().into(),
      }),
      generator_id: None,
    });

    let argument_parent = Some(Rc::downgrade(&rc));

    {
      rc.try_borrow().unwrap().body.borrow_mut().parent = argument_parent.clone().into();
    };

    let arguments = input.arguments
      .map(|x| x.arguments)
      .unwrap_or_default()
      .into_iter()
      .map(|argument| translator.parse_scope(compiler, argument, &argument_parent))
      .collect::<Result<_>>()?;

    if let Some(input) = input.return_ty {
      let return_ty = translator.parse_scope(compiler, input, parent)?;

      rc.borrow_mut().return_ty = return_ty;
    };

    {
      let mut function = rc.borrow_mut();

      function.parent = parent.clone().into();
      function.arguments = arguments;
    };

    // That dummy block from earlier gets dropped here
    rc.borrow_mut().body = FunctionBlock::parse_scope(translator, compiler, input.body, &argument_parent)?;

    Ok(rc)
  }
}
