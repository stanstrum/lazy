use super::*;

impl Scope for Function {
  type Index = str;
}

impl SearchIn<Function> for FunctionArgument {
  fn parent(&self) -> Option<RcCell<Function>> {
    Some(self.parent.clone().unwrap())
  }

  fn search_in(scope: &Function, index: &<Function as Scope>::Index) -> Result<ScopeSearch<Self, Function>> {
    Ok(
      scope.arguments.iter().find_map(|argument| (argument.borrow().name.name == index).then(|| ScopeSearch::Found(argument.clone())))
      .unwrap_or(ScopeSearch::None)
    )
  }
}

impl SearchIn<Module> for Function {
  fn parent(&self) -> Option<RcCell<Module>> {
    Some(self.parent.clone().unwrap())
  }

  fn search_in(scope: &Module, index: &<Module as Scope>::Index) -> Result<ScopeSearch<Self, Module>> {
    Ok(
      match ModuleChild::search_in(scope, index)? {
        ScopeSearch::Found(rc) => {
          match &*rc.borrow() {
            ModuleChild::Function(rc) => ScopeSearch::Found(rc.clone()),
            ModuleChild::Module(rc) => ScopeSearch::Next(rc.clone()),
          }
        },
        ScopeSearch::Next(rc) => ScopeSearch::Next(rc),
        ScopeSearch::None => ScopeSearch::None,
      }
    )
  }
}

impl<'a> ParseScope<'a> for FunctionArgument {
  type In = ast::FunctionArgument<DefaultWorkflow>;
  type Scope = Function;

  fn parse_scope(translator: &mut Translator<DefaultWorkflow>, input: Self::In, parent: &Option<RcCell<Self::Scope>>) -> Result<RcCell<Self>> {
    let module = parent.as_ref().unwrap().scope_parent();

    let ty = translator.parse_scope::<Type<Module>, Module>(input.ty, &module)?;

    Ok(new_rc_cell(Self {
      name: input.identifier,
      ty,
      parent: parent.as_ref().cloned().unwrap().into(),
    }))
  }
}

impl<'a> ParseScope<'a> for Function {
  type In = ast::Function<DefaultWorkflow>;
  type Scope = Module;

  fn parse_scope(translator: &mut Translator<DefaultWorkflow>, input: Self::In, parent: &Option<RcCell<Self::Scope>>) -> Result<RcCell<Self>> {
    let rc = new_rc_cell(Self {
      parent: parent.as_ref().unwrap().clone().into(),
      name: input.identifier,
      arguments: vec![],
      return_ty: new_rc_cell(Type::Intrinsic {
        kind: Intrinsic::Void,
        parent: parent.as_ref().cloned().unwrap().into(),
      }),
    });

    let argument_parent = Some(rc.clone());

    let arguments = input.arguments
      .map(|x| x.arguments)
      .unwrap_or_default()
      .into_iter()
      .map(|argument| translator.parse_scope::<FunctionArgument, Function>(argument, &argument_parent))
      .collect::<Result<_>>()?;

    {
      let mut function = rc.borrow_mut();

      if let Some(input) = input.return_ty {
        function.return_ty = translator.parse_scope::<Type<Module>, Module>(input, parent)?;
      };

      function.arguments = arguments;
    };

    Ok(rc)
  }
}
