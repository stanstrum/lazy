use super::*;

mod module;
mod function;

impl Resolve for Reference<Type<Module>, Module> {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    match self {
      Reference::Resolved(_) => {},
      Reference::Unresolved(rc) => {
        let search = rc.borrow().find_reference()?;

        if let ScopeSearch::Found(found) = search {
          mods.push(Modification::ResolveUnresolvedTypeModuleReference {
            weak: Rc::downgrade(rc),
            value: Self::Resolved(found.upgrade().unwrap()),
          });
        };
      },
    }; ok
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    match self {
      Reference::Resolved(rc) => rc.borrow().ensure_resolved(compiler),
      Reference::Unresolved(rc) => {
        let span = compiler.span_to_read_span(rc.borrow().span)?;

        UnresolvedQualifiedSnafu { span }.fail()?
      },
    }
  }
}

impl Resolve for Type<Module> {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    match self {
      Type::Intrinsic { .. } => ok,
      Type::Reference(reference) => reference.resolve(mods),
    }
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    match self {
      Type::Intrinsic { .. } => ok,
      Type::Reference(reference) => reference.ensure_resolved(compiler),
    }
  }
}
