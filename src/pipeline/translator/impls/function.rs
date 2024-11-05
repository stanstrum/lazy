use super::*;

impl Scope for Function {
  type Index = str;
}

impl Scope for FunctionBlock {
  type Index = str;
}

impl SearchIn<Function> for FunctionArgument {
  fn parent(&self) -> Option<WeakCell<Function>> {
    Some(self.parent.clone().unwrap())
  }

  fn search_in(scope: &Function, index: &<Function as Scope>::Index) -> Result<ScopeSearch<Self, Function>> {
    Ok(
      scope.arguments.iter()
        .find_map(|argument| {
          let name_matches = { argument.try_borrow().unwrap().name.name == index };

          name_matches.then(|| ScopeSearch::Found(Rc::downgrade(argument))
          )
      })
      .unwrap_or(ScopeSearch::None)
    )
  }
}

impl SearchIn<Module> for Function {
  fn parent(&self) -> Option<WeakCell<Module>> {
    self.parent.clone().unwrap()
  }

  fn search_in(scope: &Module, index: &<Module as Scope>::Index) -> Result<ScopeSearch<Self, Module>> {
    Ok(
      match ModuleChild::search_in(scope, index)? {
        ScopeSearch::Found(rc) => {
          match &*rc.upgrade().unwrap().try_borrow().unwrap() {
            ModuleChild::Function(rc) => ScopeSearch::Found(Rc::downgrade(rc)),
            ModuleChild::Module(rc) => ScopeSearch::Next(Rc::downgrade(rc)),
          }
        },
        ScopeSearch::Next(rc) => ScopeSearch::Next(rc),
        ScopeSearch::None => ScopeSearch::None,
      }
    )
  }
}

impl SearchIn<Function> for FunctionBlock {
  fn parent(&self) -> Option<WeakCell<Function>> {
    todo!()
  }

  fn search_in(_scope: &Function, _index: &<Function as Scope>::Index) -> Result<ScopeSearch<Self, Function>> {
    todo!()
  }
}

impl<'a> ParseScope<'a> for FunctionArgument {
  type In = ast::FunctionArgument<DefaultWorkflow>;
  type Scope = Function;

  fn parse_scope(translator: &mut Translator<DefaultWorkflow>, input: Self::In, parent: &Option<WeakCell<Self::Scope>>) -> Result<RcCell<Self>> {
    let module = { parent.clone().unwrap().upgrade().unwrap().scope_parent() };

    let ty = translator.parse_scope::<Type<Module>, Module>(input.ty, &module)?;

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

  fn parse_scope(_translator: &mut Translator<DefaultWorkflow>, _input: Self::In, _parent: &Option<WeakCell<Self::Scope>>) -> Result<RcCell<Self>> {
    todo!()
  }
}

impl<'a> ParseScope<'a> for Function {
  type In = ast::Function<DefaultWorkflow>;
  type Scope = Module;

  fn parse_scope(translator: &mut Translator<DefaultWorkflow>, input: Self::In, parent: &Option<WeakCell<Self::Scope>>) -> Result<RcCell<Self>> {
    dbg!("hello");

    let body = new_rc_cell(FunctionBlock {
      parent: None.into(),
      variables: vec![],
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
    });

    let argument_parent = Some(Rc::downgrade(&rc));

    {
      rc.try_borrow().unwrap().body.borrow_mut().parent = argument_parent.clone().into();
    };

    let arguments = input.arguments
      .map(|x| x.arguments)
      .unwrap_or_default()
      .into_iter()
      .map(|argument| translator.parse_scope(argument, &argument_parent))
      .collect::<Result<_>>()?;

    if let Some(input) = input.return_ty {
      let return_ty = translator.parse_scope(input, parent)?;

      rc.borrow_mut().return_ty = return_ty;
    };

    {
      let mut function = rc.borrow_mut();

      function.parent = parent.clone().into();
      function.arguments = arguments;
    };

    Ok(rc)
  }
}

impl SearchIn<FunctionBlock> for Variable {
  fn parent(&self) -> Option<WeakCell<FunctionBlock>> {
    todo!()
  }

  fn search_in(_scope: &FunctionBlock, _index: &<FunctionBlock as Scope>::Index) -> Result<ScopeSearch<Self, FunctionBlock>> {
    warn!("<Variable as SearchIn<FunctionBlock>>::search_in -- no one's home.");

    Ok(ScopeSearch::None)
  }
}
