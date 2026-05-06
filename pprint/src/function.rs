use super::*;

pub fn print_function_reference<'local, 'store, 'pool, C: Compiler>(
  store: &'store C::Store<'pool>,
  function_reference: &'local C::FunctionReference,
) -> String {
  let function = function_reference.rget_from(store);
  let path = store.describe_module(function.parent);
  let name = function.header.name.print(store);
  format!("{path}::{name}")
}

impl<C: Compiler> Pretty<C> for Function<C> {
  type Out = std::vec::IntoIter<String>;

  fn print<'local, 'store, 'pool>(&'local self, store: &'store C::Store<'pool>) -> Self::Out {
    let mut lines = vec![];
    let mut first: String = self.header.name.print(store);

    let ret_ty = self.header.ret_ty.print(store);
    first.push_str(format!(" -> {ret_ty}").as_str());

    lines.push(first);

    for argument in self.header.arguments.iter() {
      let ty = argument.ty.print(store);
      let name = argument.name.print(store);
      lines.push(format!("  {ty} {name}"));
    };

    lines.push("".into());
    let block = store.rget(self.body);

    for variable in block.variables.iter() {
      let ty = variable.ty.print(store);
      let name = variable.name.print(store);

      lines.push(format!("  {ty} {name} // decl"));
    };

    if !block.variables.is_empty() {
      lines.push("".into());
    };

    if !block.children.is_empty() {
      for &child in block.children.iter() {
        for line in self[child].print_with(self, store) {
          lines.push(format!("  {line}"));
        };
      };

      if !block.returns_last {
        lines.last_mut().unwrap().push(';');
      };
    } else {
      lines.push("{}".into());
    };

    lines.into_iter()
  }
}
