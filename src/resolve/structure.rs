use crate::error::{Level, MessageContents, PrintableMessage, print_message};

use crate::line_dbg;

use crate::lang::Lazy;
use crate::lang::ty::Type;
use crate::lang::reference::{AliasReference, ExpressionReference, FunctionReference, ModuleReference, Reference, TypeReference};
use crate::resolve::type_of::TypeOf;
use crate::resolve::ty::ResolvedTypePair;
use crate::resolve::coerce::{Coerce, SpecialPair};

use super::{Result, Resolve, Tasks};

impl Resolve for AliasReference {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    let reference = TypeReference::Alias(*self);
    let ty = &self.rget_from(lazy).ty;

    ResolvedTypePair(&reference, ty).resolve(lazy, tasks)
  }
}

impl Resolve for FunctionReference {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    let function = self.rget_from(lazy);
    let ret_ty = TypeReference::ReturnTypeOf(*self);

    ret_ty.resolve(lazy, tasks)?;

    let arguments_iter = (0..function.header.arguments.len())
      .map(|index| TypeReference::ArgumentOf(*self, index));

    for argument in arguments_iter {
      argument.resolve(lazy, tasks)?
    };

    let body = function.body.rget_from(lazy);

    if let Some(ty) = ret_ty.type_of(lazy)? {
      let expr_id = body.children.last().unwrap();
      let reference = TypeReference::Expression(ExpressionReference(*self, *expr_id));
      let typed_reference = Type::Reference(reference);

      let last_expression = SpecialPair(&reference, &typed_reference);
      let return_type = SpecialPair(&ret_ty, &ty);

      last_expression.coerce(lazy, &return_type, tasks)?;
    };

    unsafe {
      static mut DID_PRINT: bool = false;

      if !DID_PRINT {
        print_message(lazy, PrintableMessage {
          level: Level::Debug,
          force: false,
          description: line_dbg!("stub: resolve function").into(),
          contents: MessageContents::File(function.parent),
        });

        DID_PRINT = true;
      };
    };

    Ok(())
  }
}

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
