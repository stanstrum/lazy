use super::*;

impl<C: Compiler> Pretty<C> for lang::reference::BlockReference<C> {
  type Out = String;

  fn print<'local, 'store, 'pool>(&self, store: &'store C::Store<'pool>) -> Self::Out {
    let Span { start, end, .. } = self.rget_from(store).span;
    format!("block {}:{} - {}:{}", start.line, start.column, end.line, end.column)
  }
}

impl<C: Compiler> Pretty<C> for FunctionAnd<'_, C, lang::expr::BlockExpression<C>> {
  type Out = std::vec::IntoIter<String>;

  fn print<'local, 'store, 'pool>(&'local self, store: &'store C::Store<'pool>) -> Self::Out {
    let (function, block) = self;
    let mut lines = vec!["{".into()];

    for variable in block.variables.iter() {
      let ty = variable.ty.print(store);
      let name = variable.name.print(store);

      lines.push(format!("  {ty} {name}"));
    };

    if !block.variables.is_empty() {
      lines.push("".into());
    };

    for &child in block.children.iter() {
      for line in function[child].print_with(function, store) {
        lines.push(format!("  {line}"));
      };
    };

    if lines.len() == 1 {
      lines.first_mut().unwrap().push('}');
    } else {
      if !block.returns_last {
        lines.last_mut().unwrap().push(';');
      };

      lines.push("}".into());
    };

    lines.into_iter()
  }
}
