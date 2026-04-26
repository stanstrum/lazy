use lang::ty::Type;
use lang::{Compiler, CompilerPoolStore};
use lang::reference::{AliasReference, StructReference, TypeReference};
use crate::impls::function::resolve_function_reference;
use crate::impls::ty::verify_typeof;

use super::*;

pub(crate) fn resolve_module_reference<C: Compiler + 'static>(store: &C::Store<'_>, module_reference: &C::ModuleReference, tasks: &mut Tasks<C>) -> Result<C> {
  let module = module_reference.rget_from(store);

  for module in module.modules.iter() {
    resolve_module_reference(store, module, tasks)?;
  };

  for function in module.functions.iter() {
    resolve_function_reference(store, function, tasks)?;
  };

  for index in 0..module.aliases.len() {
    AliasReference(*module_reference, index).resolve(store, tasks)?;
  };

  for index in 0..module.structs.len() {
    StructReference(*module_reference, index).resolve(store, tasks)?;
  };

  Ok(())
}

impl<C: Compiler + 'static> Resolve<C> for AliasReference<C> {
  fn resolve(&self, store: &C::Store<'_>, tasks: &mut Tasks<C>) -> Result<C> {
    TypeReference::Alias(*self).resolve(store, tasks)
  }
}

impl<C: Compiler + 'static> Resolve<C> for StructReference<C> {
  fn resolve(&self, store: &C::Store<'_>, tasks: &mut Tasks<C>) -> Result<C> {
    let struct_borrow = self.rget_from(store);

    for index in 0..struct_borrow.members.len() {
      TypeReference::StructMember(*self, index).resolve(store, tasks)?;
    };

    Ok(())
  }
}

pub(super) fn verify_struct<C: Compiler + 'static>(store: &C::Store<'_>, struct_reference: &StructReference<C>, tasks: &mut Tasks<C>) -> Result<C> {
  let struct_borrow = struct_reference.rget_from(store);

  let module_name = store.describe_module(struct_reference.0);
  let struct_name = struct_borrow.name.print(store);

  for (id, field_name) in struct_borrow.members.iter().map(|x| x.name.print(store)).enumerate() {
    let description = format!("Verifying struct field type {module_name}::{struct_name}::{field_name}");
    let type_reference = TypeReference::StructMember(*struct_reference, id);

    tasks.work(description, |tasks| ty::verify_typeof(store, &type_reference, tasks))?;
  };

  Ok(())
}

fn verify_alias<C: Compiler + 'static>(store: &C::Store<'_>, alias: &AliasReference<C>, tasks: &mut Tasks<C>) -> Result<C> {
  let ty = Type::Reference(TypeReference::Alias(*alias));
  verify_typeof(store, &ty, tasks)
}

pub(crate) fn default_types_in_module<C: Compiler + 'static>(store: &mut C::Store<'_>, module: &C::ModuleReference, tasks: &mut Tasks<C>) -> Result<C> {
  // let description = {
  let borrow = module.rget_from(store);

  let modules = borrow.modules.clone();
  let functions = borrow.functions.clone();

  for id in 0..borrow.aliases.len() {
    let reference = AliasReference(*module, id);
    ty::default_types_of_type(store, &TypeReference::Alias(reference), tasks)?;
  };

  for module in modules {
    default_types_in_module(store, &module, tasks)?;
  };

  for function in functions {
    function::default_types_in_function(store, &function, tasks)?;
  };

  Ok(())
}

pub(crate) fn verify_module<C: Compiler + 'static>(store: &C::Store<'_>, module: &C::ModuleReference, tasks: &mut Tasks<C>) -> Result<C> {
  let borrow = module.rget_from(store);

  for id in 0..borrow.aliases.len() {
    let reference = AliasReference(*module, id);
    verify_alias(store, &reference, tasks)?;
  };

  for module in borrow.modules.iter() {
    verify_module(store, module, tasks)?;
  };

  for function in borrow.functions.iter() {
    function::verify_function(store, function, tasks)?;
  };

  for id in 0..borrow.structs.len() {
    let reference = StructReference(*module, id);
    verify_struct(store, &reference, tasks)?;
  };

  Ok(())
}
