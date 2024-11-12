use super::*;

pub(in crate::pipeline::checker) trait GetAndMaybeModify<V: SearchIn<S>, S: Scope> {
  fn get_and_maybe_modify(&self, mods: &mut Modifications) -> Result<Option<WeakCell<V>>>;
}

pub(crate) trait MakeModification<S: Scope, W: CompilerWorkflow = DefaultWorkflow> {
  fn make_resolve_reference(_this: &RcCell<Reference<Self, S>>, _value: &WeakCell<Self>) -> Modification where Self: SearchIn<S>;
}

impl<V: SearchIn<S>, S: Scope> GetAndMaybeModify<V, S> for RcCell<Reference<V, S>> {
  fn get_and_maybe_modify(&self, mods: &mut Modifications) -> Result<Option<WeakCell<V>>> {
    Ok(match &*self.borrow() {
      Reference::Resolved(rc) => Some(Rc::downgrade(rc)),
      Reference::Unresolved(unresolved) => {
        let name = enchant!("get_and_maybe_modify");

        if let ScopeSearch::Found(weak) = unresolved.borrow().find_reference()? {
          trace!("{name}: resolved a reference");
          mods.push(V::make_resolve_reference(self, &weak));
          Some(weak)
        } else {
          trace!("{name}: couldn't resolve a reference");
          None
        }
      },
    })
  }
}

// impl Resolve for RcCell<Reference<ModuleChild, Module>> {
//   fn resolve(&self, mods: &mut Modifications) -> Result {
//     if let Some(weak) = self.get_and_maybe_modify(mods)?.as_ref().and_then(Weak::upgrade) {
//       weak.borrow().resolve(mods)?;
//     } else {
//       warn!("{}: couldn't resolve because the reference couldn't be resolved", enchant!("resolve"));
//     };

//     ok
//   }

//   fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
//     match &*self.borrow() {
//       Reference::Resolved(rc) => rc.ensure_resolved(compiler),
//       Reference::Unresolved(rc) => {
//         let span = rc.borrow().span;
//         let span = compiler.span_to_read_span(span)?;
//         return UnresolvedQualifiedSnafu { span }.fail()?;
//       },
//     }
//   }
// }

impl Resolve for TypeAlias {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    self.ty.resolve(mods)
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    self.ty.ensure_resolved(compiler)
  }
}

impl Resolve for ModuleChild {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    match self {
      ModuleChild::Function(rc) => rc.resolve(mods),
      ModuleChild::Module(rc) => rc.resolve(mods),
      ModuleChild::Type(rc) => rc.resolve(mods),
    }
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    match self {
      ModuleChild::Function(rc) => rc.ensure_resolved(compiler),
      ModuleChild::Module(rc) => rc.ensure_resolved(compiler),
      ModuleChild::Type(rc) => rc.ensure_resolved(compiler),
    }
  }
}

impl Resolve for RcCell<Module> {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    let this = self.borrow();

    for import in this.imports.iter() {
      import.reference.resolve(mods)?;
    };

    for export in this.exports.iter() {
      export.get_reference().upgrade().unwrap().resolve(mods)?;
    };

    for child in this.children.iter() {
      child.borrow().resolve(mods)?;
    };

    ok
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    let this = self.borrow();

    for import in this.imports.iter() {
      import.reference.ensure_resolved(compiler)?;
    };

    for export in this.exports.iter() {
      export.get_reference().upgrade().unwrap().ensure_resolved(compiler)?;
    };

    for child in this.children.iter() {
      child.borrow().ensure_resolved(compiler)?;
    };

    ok
  }
}

impl MakeModification<Module> for Export {
  fn make_resolve_reference(_this: &RcCell<Reference<Self, Module>>, _value: &WeakCell<Self>) -> Modification where Self: SearchIn<Module> {
    todo!()
  }
}

impl MakeModification<Module> for Type<Module> {
  fn make_resolve_reference(this: &RcCell<Reference<Self, Module>>, value: &WeakCell<Self>) -> Modification where Self: SearchIn<Module> {
    Modification::ResolveUnresolvedTypeModuleReference {
      weak: Rc::downgrade(this),
      value: Reference::Resolved(value.upgrade().unwrap()),
    }
  }
}

impl MakeModification<Module> for TypeAlias {
  fn make_resolve_reference(_this: &RcCell<Reference<Self, Module>>, _value: &WeakCell<Self>) -> Modification where Self: SearchIn<Module> {
    todo!()
  }
}

impl MakeModification<Module> for Function {
  fn make_resolve_reference(_this: &RcCell<Reference<Self, Module>>, _value: &WeakCell<Self>) -> Modification where Self: SearchIn<Module> {
    todo!()
  }
}

impl MakeModification<Module> for ModuleChild {
  fn make_resolve_reference(_this: &RcCell<Reference<Self, Module>>, _value: &WeakCell<Self>) -> Modification where Self: SearchIn<Module> {
    todo!()
  }
}

impl MakeModification<Module> for Module {
  fn make_resolve_reference(_this: &RcCell<Reference<Self, Module>>, _value: &WeakCell<Self>) -> Modification where Self: SearchIn<Module> {
    todo!()
  }
}

impl MakeModification<Function> for FunctionArgument {
  fn make_resolve_reference(_this: &RcCell<Reference<Self, Function>>, _value: &WeakCell<Self>) -> Modification where Self: SearchIn<Function> {
    todo!()
  }
}

impl MakeModification<Function> for FunctionBlock {
  fn make_resolve_reference(_this: &RcCell<Reference<Self, Function>>, _value: &WeakCell<Self>) -> Modification where Self: SearchIn<Function> {
    todo!()
  }
}

impl MakeModification<FunctionBlock> for Variable {
  fn make_resolve_reference(_this: &RcCell<Reference<Self, FunctionBlock>>, _value: &WeakCell<Self>) -> Modification where Self: SearchIn<FunctionBlock> {
    todo!()
  }
}

impl MakeModification<FunctionBlock> for Instruction {
  fn make_resolve_reference(_this: &RcCell<Reference<Self, FunctionBlock>>, _value: &WeakCell<Self>) -> Modification where Self: SearchIn<FunctionBlock> {
    todo!()
  }
}
