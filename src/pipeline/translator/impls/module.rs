use super::*;

impl Scope for Module {
  type Index = str;
}

impl SearchIn<Module> for ModuleChild {
  fn parent(&self) -> Option<WeakCell<Module>> {
    match self {
      ModuleChild::Function(rc) => rc.try_borrow().unwrap().parent(),
      ModuleChild::Module(rc) => rc.try_borrow().unwrap().parent(),
    }
  }

  fn search_in(scope: &Module, index: &<Module as Scope>::Index) -> Result<ScopeSearch<Self, Module>> {
    Ok(
      scope.children.iter().find_map(|child| (
        match &*child.try_borrow().unwrap() {
          ModuleChild::Function(rc) => {
            let name_matches = rc.try_borrow().unwrap().name.name == index;

            name_matches.then(|| ScopeSearch::Found(Rc::downgrade(child)))
          },
          ModuleChild::Module(rc) => (rc.try_borrow().unwrap().name == index).then(|| ScopeSearch::Next(Rc::downgrade(rc))),
        }
      )).unwrap_or(ScopeSearch::None)
    )
  }
}

impl SearchIn<Module> for Module {
  fn parent(&self) -> Option<WeakCell<Module>> {
    self.parent.clone().unwrap()
  }

  fn search_in(scope: &Module, index: &<Module as Scope>::Index) -> Result<ScopeSearch<Self, Module>> {
    Ok(
      match ModuleChild::search_in(scope, index)? {
        ScopeSearch::Found(rc) => {
          match &*rc.upgrade().unwrap().try_borrow().unwrap() {
            ModuleChild::Module(rc) => ScopeSearch::Next(Rc::downgrade(rc)),
            _ => ScopeSearch::None,
          }
        },
        ScopeSearch::Next(rc) => ScopeSearch::Next(rc),
        ScopeSearch::None => ScopeSearch::None,
      }
    )
  }
}

impl<'a> ParseScope<'a> for ModuleChild {
  type In = ast::NamespaceChild<DefaultWorkflow>;
  type Scope = Module;

  fn parse_scope(translator: &mut Translator<DefaultWorkflow>, input: Self::In, parent: &Option<WeakCell<Self::Scope>>) -> Result<RcCell<Self>> {
    match input {
      ast::NamespaceChild::Namespace(namespace) => {
        let module = translator.parse_scope(*namespace, parent)?;

        Ok(new_rc_cell(Self::Module(module)))
      },
      ast::NamespaceChild::Function(function) => {
        let function = translator.parse_scope(function, parent)?;

        Ok(new_rc_cell(Self::Function(function)))
      },
      ast::NamespaceChild::Alias(_) => todo!(),
    }
  }
}

impl<'a> ParseScope<'a> for Module {
  type In = ast::Namespace<DefaultWorkflow>;
  type Scope = Module;

  fn parse_scope(translator: &mut Translator<DefaultWorkflow>, input: Self::In, parent: &Option<WeakCell<Self::Scope>>) -> Result<RcCell<Self>> {
    let module = new_rc_cell(Self {
      parent: parent.clone().into(),
      name: ModuleName::Identifier(input.identifier),
      children: vec![],
      span: input.span,
      imports: vec![],
      exports: vec![],
      generator_id: None,
    });

    let child_parent = Some(Rc::downgrade(&module));

    let children = input.children.into_iter()
      .map(|child| translator.parse_scope::<ModuleChild, Self::Scope>(child, &child_parent))
      .collect::<Result<_>>()?;

    {
      module.borrow_mut().children = children;
    };

    Ok(module)
  }
}
