use crate::lang::ty::Type;
use crate::lang::reference::{AliasReference, ExpressionReference, FunctionReference, ModuleReference, TypeReference, VariableReference};
use crate::resolve::TypePair;

use super::*;

impl Resolve for ModuleReference {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    let module = self.rget_from(lazy);

    for module in module.modules.iter() {
      module.resolve(lazy, tasks)?;
    };

    for function in module.functions.iter() {
      function.resolve(lazy, tasks)?;
    };

    for index in 0..module.aliases.len() {
      let reference = AliasReference(*self, index);
      reference.resolve(lazy, tasks)?;
    };

    Ok(())
  }
}

impl Resolve for AliasReference {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    let reference = TypeReference::Alias(*self);
    let ty = &self.rget_from(lazy).ty;

    TypePair::new(&reference, ty).resolve(lazy, tasks)
  }
}

impl Resolve for FunctionReference {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    let description = format!(line_dbg!("Resolve FunctionReference: {}"), self.print(lazy));

    tasks.work(description, |tasks|{
      let function = self.rget_from(lazy);
      let ret_ty = TypeReference::ReturnTypeOf(*self);

      ret_ty.resolve(lazy, tasks)?;

      let arguments_iter = (0..function.header.arguments.len())
        .map(|index| TypeReference::Variable(VariableReference::Argument(*self, index)));

      for argument in arguments_iter {
        argument.resolve(lazy, tasks)?;
      };

      let body = lazy.rget(function.body);
      if let Some(ty) = ret_ty.type_of(lazy)? {
        let expr_id = body.children.last().unwrap();
        let reference = TypeReference::Expression(ExpressionReference(function.body, *expr_id));
        let typed_reference = Type::Reference(reference);

        let last_expression = TypePair::new(&reference, &typed_reference);
        let return_type: TypePair = TypePair::new(&ret_ty, &ty);

        last_expression.coerce(lazy, &return_type, tasks)?;
      };

      function.body.resolve(lazy, tasks)?;

      Ok(())
    })
  }
}
