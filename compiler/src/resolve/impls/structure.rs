use crate::lang::ty::Type;
use lang::CompilerPoolStore;
use ::lang::reference::{AliasReference, ExpressionReference, StructReference};
use gluezy::{FunctionReference, Lazy, LazyStructures, ModuleReference, TypeReference, VariableReference};
use crate::resolve::TypePair;
use crate::resolve::impls::ty::verify_typeof;

use super::*;

impl Resolve for ModuleReference {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks<LazyStructures>) -> Result<()> {
    let module = self.rget_from(lazy);

    for module in module.modules.iter() {
      module.resolve(lazy, tasks)?;
    };

    for function in module.functions.iter() {
      function.resolve(lazy, tasks)?;
    };

    for index in 0..module.aliases.len() {
      AliasReference(*self, index).resolve(lazy, tasks)?;
    };

    for index in 0..module.structs.len() {
      StructReference(*self, index).resolve(lazy, tasks)?;
    };

    Ok(())
  }
}

impl Resolve for AliasReference<LazyStructures> {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks<LazyStructures>) -> Result<()> {
    TypeReference::Alias(*self).resolve(lazy, tasks)
  }
}

impl Resolve for StructReference<LazyStructures> {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks<LazyStructures>) -> Result<()> {
    let struct_borrow = self.rget_from(lazy);

    for index in 0..struct_borrow.members.len() {
      TypeReference::StructMember(*self, index).resolve(lazy, tasks)?;
    };

    Ok(())
  }
}

pub(super) fn verify_struct(lazy: &Lazy, struct_reference: &StructReference<LazyStructures>, tasks: &mut Tasks<LazyStructures>) -> Result<()> {
  let struct_borrow = struct_reference.rget_from(lazy);

  let module_name = lazy.describe_module(struct_reference.0);
  let struct_name = struct_borrow.name.print(lazy);

  for (id, field_name) in struct_borrow.members.iter().map(|x| x.name.print(lazy)).enumerate() {
    let description = format!("Verifying struct field type {module_name}::{struct_name}::{field_name}");
    let type_reference = TypeReference::StructMember(*struct_reference, id);

    tasks.work(description, |tasks| ty::verify_typeof(lazy, &type_reference, tasks))?;
  };

  Ok(())
}

impl Resolve for FunctionReference {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks<LazyStructures>) -> Result<()> {
    let description = format!(line_dbg!("Resolve FunctionReference: {}"), pprint::print_function_reference(self, lazy));

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
      if let Some(ty) = ret_ty.type_of(lazy) {
        let expr_id = body.children.last().unwrap();
        let reference = TypeReference::Expression(ExpressionReference(function.body, *expr_id));
        let typed_reference = Type::Reference(reference);

        let last_expression = TypePair::new(reference, typed_reference);
        let return_type: TypePair<LazyStructures> = TypePair::new(ret_ty, ty);

        last_expression.coerce(lazy, &return_type, tasks)?;
      };

      function.body.resolve(lazy, tasks)?;

      Ok(())
    })
  }
}

fn verify_alias(lazy: &Lazy, alias: &AliasReference<LazyStructures>, tasks: &mut Tasks<LazyStructures>) -> Result<()> {
  let ty = Type::Reference(TypeReference::Alias(*alias));
  verify_typeof(lazy, &ty, tasks)
}

pub(in crate::resolve) fn default_types_in_module(lazy: &mut Lazy, module: &ModuleReference, tasks: &mut Tasks<LazyStructures>) -> Result<()> {
  // let description = {
  let borrow = module.rget_from(lazy);

  let modules = borrow.modules.clone();
  let functions = borrow.functions.clone();

  for id in 0..borrow.aliases.len() {
    let reference = AliasReference(*module, id);
    ty::default_types_of_type(lazy, &TypeReference::Alias(reference), tasks)?;
  };

  for module in modules {
    default_types_in_module(lazy, &module, tasks)?;
  };

  for function in functions {
    function::default_types_in_function(lazy, &function, tasks)?;
  };

  Ok(())
}

pub(in crate::resolve) fn verify_module(lazy: &Lazy, module: &ModuleReference, tasks: &mut Tasks<LazyStructures>) -> Result<()> {
  let borrow = module.rget_from(lazy);

  for id in 0..borrow.aliases.len() {
    let reference = AliasReference(*module, id);
    verify_alias(lazy, &reference, tasks)?;
  };

  for module in borrow.modules.iter() {
    verify_module(lazy, module, tasks)?;
  };

  for function in borrow.functions.iter() {
    function::verify_function(lazy, function, tasks)?;
  };

  for id in 0..borrow.structs.len() {
    let reference = StructReference(*module, id);
    verify_struct(lazy, &reference, tasks)?;
  };

  Ok(())
}
