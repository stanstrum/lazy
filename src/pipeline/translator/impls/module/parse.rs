use super::*;

impl<'a> ParseScope<'a> for TypeAlias {
  type In = ast::TypeAlias<DefaultWorkflow>;
  type Scope = Module;

  fn parse_scope(
    translator: &mut Translator<DefaultWorkflow>,
    compiler: &Compiler<DefaultWorkflow>,
    input: Self::In,
    parent: &Option<WeakCell<Self::Scope>>,
  ) -> Result<RcCell<Self>> {
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
    parent: &Option<WeakCell<Module>>,
  ) -> Result<Vec<Export>> {
    match input {
      ast::Export::NamespaceChild(child) => {
        let child = translator.parse_scope::<ModuleChild, _>(compiler, *child, parent)?;
        let name = { child.borrow().name(compiler) };

        Ok(vec![Export {
          name,
          reference: new_rc_cell(Reference::Resolved(child)),
        }])
      },
      ast::Export::Import(_) => todo!(),
    }
  }

  pub(in crate::pipeline::translator) fn parse_exports(
    translator: &mut Translator<DefaultWorkflow>,
    compiler: &Compiler<DefaultWorkflow>,
    input: Vec<ast::Export<DefaultWorkflow>>,
    parent: &Option<WeakCell<Module>>,
  ) -> Result<Vec<Export>> {
    Ok(
      input
        .into_iter()
        .map(|input| Export::parse_export(translator, compiler, input, parent))
        .collect::<Result<Vec<Vec<Export>>>>()?
        .into_iter()
        .flatten()
        .collect(),
    )
  }
}

impl<'a> ParseScope<'a> for ModuleChild {
  type In = ast::NamespaceChild<DefaultWorkflow>;
  type Scope = Module;

  fn parse_scope(
    translator: &mut Translator<DefaultWorkflow>,
    compiler: &Compiler<DefaultWorkflow>,
    input: Self::In,
    parent: &Option<WeakCell<Self::Scope>>,
  ) -> Result<RcCell<Self>> {
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

  fn parse_scope(
    translator: &mut Translator<DefaultWorkflow>,
    compiler: &Compiler<DefaultWorkflow>,
    input: Self::In,
    parent: &Option<WeakCell<Self::Scope>>,
  ) -> Result<RcCell<Self>> {
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
          let child = translator.parse_scope(compiler, other, &child_parent)?;
          children.push(child);
        },
      };
    }

    {
      module.borrow_mut().children = children;
      module.borrow_mut().exports = exports;
    };

    Ok(module)
  }
}
