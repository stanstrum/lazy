use super::*;

impl SearchIn<Module> for TypeAlias {}
impl SearchIn<Module> for Export {}
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
