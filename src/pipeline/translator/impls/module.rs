use super::*;

impl Scope for Module {
  type Index = str;
}

impl SearchIn<Module> for ModuleChild {
  fn parent(&self) -> Option<RcCell<Module>> {
    match self {
      ModuleChild::Function(rc) => rc.borrow().parent(),
      ModuleChild::Module(rc) => rc.borrow().parent(),
    }
  }

  fn search_in(scope: &Module, index: &<Module as Scope>::Index) -> Result<ScopeSearch<Self, Module>> {
    Ok(
      scope.children.iter().find_map(|child| (
        match &*child.borrow() {
          ModuleChild::Function(rc) => {
            let name_matches = rc.borrow().name.name == index;

            name_matches.then(|| ScopeSearch::Found(new_rc_cell(ModuleChild::Function(rc.clone()))))
          },
          ModuleChild::Module(rc) => (rc.borrow().name == index).then(|| ScopeSearch::Next(rc.clone())),
        }
      )).unwrap_or(ScopeSearch::None)
    )
  }
}

impl SearchIn<Module> for Module {
  fn parent(&self) -> Option<RcCell<Module>> {
    self.parent.clone().unwrap()
  }

  fn search_in(scope: &Module, index: &<Module as Scope>::Index) -> Result<ScopeSearch<Self, Module>> {
    Ok(
      match ModuleChild::search_in(scope, index)? {
        ScopeSearch::Found(rc) => {
          match &*rc.borrow() {
            ModuleChild::Module(rc) => ScopeSearch::Next(rc.clone()),
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

  fn parse_scope(translator: &mut Translator<DefaultWorkflow>, input: Self::In, parent: &Option<RcCell<Self::Scope>>) -> Result<RcCell<Self>> {
    match input {
      ast::NamespaceChild::Namespace(namespace) => {
        let module  = translator.parse_scope(*namespace, parent)?;

        Ok(new_rc_cell(Self::Module(module)))
      },
      ast::NamespaceChild::Function(function) => {
        let function  = translator.parse_scope(function, parent)?;

        Ok(new_rc_cell(Self::Function(function)))
      },
    }
  }
}

impl<'a> ParseScope<'a> for Module {
  type In = ast::Namespace<DefaultWorkflow>;
  type Scope = Module;

  fn parse_scope(translator: &mut Translator<DefaultWorkflow>, input: Self::In, parent: &Option<RcCell<Self::Scope>>) -> Result<RcCell<Self>> {
    let module = new_rc_cell(Self {
      parent: parent.clone().into(),
      name: ModuleName::Identifier(input.identifier),
      children: vec![],
      span: input.span,
    });

    let child_parent = Some(module.clone());

    let children = input.children.into_iter()
      .map(|child| translator.parse_scope::<ModuleChild, Self::Scope>(child, &child_parent))
      .collect::<Result<_>>()?;

    {
      module.borrow_mut().children = children;
    };

    Ok(module)
  }
}
