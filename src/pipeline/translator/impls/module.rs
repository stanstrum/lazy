use crate::enchant;

use super::*;

impl Scope for Module {
  type Index = str;
  type Part = ast::Identifier<DefaultWorkflow>;
}

impl SearchIn<Module> for TypeAlias {
  fn parent(&self) -> Option<WeakCell<Module>> {
    todo!()
  }

  fn search_in(scope: &Module, index: &<Module as Scope>::Index) -> Result<ScopeSearch<Self, Module>> {
    todo!()
  }
}

impl SearchIn<Module> for ModuleChild {
  fn parent(&self) -> Option<WeakCell<Module>> {
    match self {
      ModuleChild::Function(rc) => rc.scope_parent(),
      ModuleChild::Module(rc) => rc.scope_parent(),
      ModuleChild::Type(rc) => rc.scope_parent(),
    }
  }

  fn search_in(scope: &Module, index: &<Module as Scope>::Index) -> Result<ScopeSearch<Self, Module>> {
    for child in scope.children.iter() {
      return Ok(
        match &*child.borrow() {
          ModuleChild::Function(function) if function.borrow().name.name == index => {
            ScopeSearch::Found(Rc::downgrade(child))
          },
          ModuleChild::Type(alias) if alias.borrow().name.name == index => {
            ScopeSearch::Found(Rc::downgrade(child))
          },
          ModuleChild::Module(module) if module.borrow().name == index => {
            ScopeSearch::Next(Rc::downgrade(module))
          },
          _ => continue,
        }
      );
    };

    for import in scope.imports.iter() {
      if import.name == index {
        return Ok(match import.reference.get()? {
          Some(weak) => {
            ScopeSearch::Found(weak)
          },
          None => {
            warn!("{}: resolved reference to import, but import is unresolved", enchant!("search_in"));
            ScopeSearch::None
          },
        });
      };
    };

    for export in scope.exports.iter() {
      if export.name.as_ref().is_some_and(|name| name == index) {
        return Ok(match export.reference.borrow().get()? {
          Some(weak) => {
            ScopeSearch::Found(weak)
          },
          None => {
            warn!("{}: resolved reference to export, but export is unresolved", enchant!("search_in"));
            ScopeSearch::None
          },
        });
      };
    };

    Ok(ScopeSearch::None)
  }
}

impl SearchIn<Module> for Export {
  fn parent(&self) -> Option<WeakCell<Module>> {
    todo!()
  }

  fn search_in(scope: &Module, index: &<Module as Scope>::Index) -> Result<ScopeSearch<Self, Module>> {
    todo!()
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

impl<'a> ParseScope<'a> for TypeAlias {
  type In = ast::TypeAlias<DefaultWorkflow>;
  type Scope = Module;

  fn parse_scope(translator: &mut Translator<DefaultWorkflow>, compiler: &Compiler<DefaultWorkflow>, input: Self::In, parent: &Option<WeakCell<Self::Scope>>) -> Result<RcCell<Self>> {
    let ty = translator.parse_scope(compiler, input.ty, parent)?;

    Ok(new_rc_cell(Self {
      parent: parent.clone().unwrap().into(),
      name: input.name,
      ty,
    }))
  }
}

impl Export {
  pub(in crate::pipeline::translator) fn parse_export(
    translator: &mut Translator<DefaultWorkflow>,
    compiler: &Compiler<DefaultWorkflow>,
    input: ast::Export<DefaultWorkflow>,
    parent: &Option<WeakCell<Module>>
  ) -> Result<Vec<Export>> {
    match input {
      ast::Export::NamespaceChild(child) => {
        let child = translator.parse_scope::<ModuleChild, Module>(compiler, *child, parent)?;
        let name = { child.borrow().name(compiler) };

        Ok(vec![Export {
          name,
          reference: new_rc_cell(Reference::Resolved(child)),
        }])
      },
      ast::Export::Import(import) => todo!(),
    }
  }

  pub(in crate::pipeline::translator) fn parse_exports(
    translator: &mut Translator<DefaultWorkflow>,
    compiler: &Compiler<DefaultWorkflow>,
    input: Vec<ast::Export<DefaultWorkflow>>,
    parent: &Option<WeakCell<Module>>
  ) -> Result<Vec<Export>> {
     Ok(
      input.into_iter()
        .map(|input| Export::parse_export(translator, compiler, input, parent))
        .collect::<Result<Vec<Vec<Export>>>>()?
        .into_iter()
        .flatten()
        .collect()
    )
  }
}

impl<'a> ParseScope<'a> for ModuleChild {
  type In = ast::NamespaceChild<DefaultWorkflow>;
  type Scope = Module;

  fn parse_scope(translator: &mut Translator<DefaultWorkflow>, compiler: &Compiler<DefaultWorkflow>, input: Self::In, parent: &Option<WeakCell<Self::Scope>>) -> Result<RcCell<Self>> {
    match input {
      ast::NamespaceChild::Namespace(namespace) => {
        let module = translator.parse_scope(compiler, *namespace, parent)?;
        Ok(new_rc_cell(Self::Module(module)))
      },
      ast::NamespaceChild::Function(function) => {
        let function = translator.parse_scope(compiler, function, parent)?;
        Ok(new_rc_cell(Self::Function(function)))
      },
      ast::NamespaceChild::Alias(alias) => {
        let alias = translator.parse_scope(compiler, alias, parent)?;
        Ok(new_rc_cell(Self::Type(alias)))
      },
      ast::NamespaceChild::Export(_) => unimplemented!(),
    }
  }
}

impl<'a> ParseScope<'a> for Module {
  type In = ast::Namespace<DefaultWorkflow>;
  type Scope = Module;

  fn parse_scope(translator: &mut Translator<DefaultWorkflow>, compiler: &Compiler<DefaultWorkflow>, input: Self::In, parent: &Option<WeakCell<Self::Scope>>) -> Result<RcCell<Self>> {
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

    let mut children = vec![];
    let mut exports = vec![];

    for child in input.children {
      match child {
        ast::NamespaceChild::Export(export) => {
          let parsed = Export::parse_export(translator, compiler, export, &child_parent)?;
          exports.extend(parsed);
        },
        other => {
          let child = translator.parse_scope::<ModuleChild, Module>(compiler, other, &child_parent)?;
          children.push(child);
        },
      };

      todo!()
    };

    // let children = input.children.into_iter()
    //   .map(|child| translator.parse_scope::<ModuleChild, Self::Scope>(child, &child_parent))
    //   .collect::<Result<_>>()?;

    {
      module.borrow_mut().children = children;
      module.borrow_mut().exports = exports;
    };

    Ok(module)
  }
}


impl Export {
  pub(crate) fn get_reference(&self) -> WeakCell<Reference<ModuleChild, Module>> {
    Rc::downgrade(&self.reference)
  }

  pub(crate) fn get_name(&self, compiler: &Compiler<DefaultWorkflow>) -> Result<Option<String>> {
    if let Some(name) = self.name.to_owned() {
      return Ok(Some(name));
    };

    if let Some(weak) = self.get_reference().upgrade().unwrap().get()? {
      let Some(rc) = weak.upgrade() else {
        return Ok(None);
      };

      return Ok(rc.borrow().name(compiler));
    };

    Ok(None)
  }
}

impl ModuleName {
  pub(crate) fn name(&self, compiler: &Compiler<DefaultWorkflow>) -> String {
    match self {
      ModuleName::Identifier(identifier) => identifier.name.to_owned(),
      ModuleName::File(handle) => handle.proper_name(compiler),
    }
  }
}

impl ModuleChild {
  pub(crate) fn name(&self, compiler: &Compiler<DefaultWorkflow>) -> Option<String> {
    match self {
      ModuleChild::Function(rc) => Some(rc.borrow().name.name.to_owned()),
      ModuleChild::Type(rc) => Some(rc.borrow().name.name.to_owned()),
      ModuleChild::Module(rc) => Some(rc.borrow().name.name(compiler)),
    }
  }
}
