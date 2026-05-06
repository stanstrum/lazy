use super::*;

impl<C: Compiler> Pretty<C> for lang::module::Name<C> {
  type Out = String;

  fn print<'local, 'store, 'pool>(&'local self, store: &'store C::Store<'pool>) -> Self::Out {
    store.pool().get(self.id)
  }
}

impl<C: Compiler> Pretty<C> for lang::module::Module<C> {
  type Out = std::vec::IntoIter<String>;

  fn print<'local, 'store, 'pool>(&self, store: &'store C::Store<'pool>) -> Self::Out {
    let name = store.pool().get(self.name);
    let mut lines = vec![
      format!("mod {name}")
    ];

    let mut needs_empty = false;

    for (key, value) in self.transports.import_map.iter() {
      if needs_empty {
        lines.push("".into());
      };

      lines.push(format!("  import from {:?}", value.implicit.print(store)));
      lines.push(format!("    {} // {}", store.pool().get(*key), value.print(store)));

      needs_empty = true;
    };

    for (space, _) in self.transports.import_stars.iter() {
      if needs_empty {
        lines.push("".into());
      };

      lines.push(format!("  import from {}", space.print(store)));
      lines.push("    *".into());

      needs_empty = true;
    };

    if needs_empty {
      lines.push("".into());
      needs_empty = false;
    };

    for alias in self.aliases.iter() {
      let name = alias.name.print(store);
      let ty = alias.ty.print(store);
      lines.push(format!("  type {name} := {ty}"));

      needs_empty = true;
    };

    for struc in self.structs.iter() {
      if needs_empty {
        lines.push("".into());
      };

      let name = struc.name.print(store);

      lines.push(format!("  struct {name}"));

      for member in struc.members.iter() {
        lines.push(format!("    {} {}", member.ty.print(store), member.name.print(store)));
      };

      needs_empty = true;
    };

    for &module in self.modules.iter() {
      let module_borrow = store.rget(module);

      if needs_empty {
        lines.push("".into());
      };

      lines.push(format!("  // {}", store.describe_module(module)));
      for line in module_borrow.print(store) {
        lines.push(format!("  {line}"));
      };

      needs_empty = true;
    };

    for function in self.functions.iter() {
      let function_borrow = function.rget_from(store);

      if needs_empty {
        lines.push("".into());
      };

      for line in function_borrow.print(store) {
        lines.push(format!("  {line}"));
      };

      needs_empty = true;
    };

    lines.into_iter()
  }
}
