pub mod span;
pub mod token;
pub mod intrinsic;

pub mod module;
pub mod function;
pub mod import;

pub mod ty;
pub mod reference;

pub mod expr;

mod store;

use std::{fmt::Debug, hash::Hash};

use string_pool::StringPool;

use crate::module::{ModuleParent, ModulePath};

pub trait CompilerReference: Debug + Clone + Copy + PartialEq + Eq {}
impl<T: Debug + Clone + Copy + PartialEq + Eq> CompilerReference for T {}

pub trait CompilerPoolStore<C: Compiler>:
  reference::Store<C::ModuleReference, Out = module::Module<C>> +
  reference::Store<C::FunctionReference, Out = function::Function<C>> +
  reference::Store<C::TokensReference, Out = token::Tokens<C>>
{
  fn pool(&self) -> &StringPool;

  fn describe_module(&self, module_reference: C::ModuleReference) -> String {
    let module = self.rget(module_reference);
    let name = self.pool().get(module.name);

    match &module.parent {
      ModuleParent::Path(ModulePath { /* path, */ .. }) => {
        // let mut path = path.as_path();

        // if
        //   let Some(parent) = self.settings.input_path.parent() &&
        //   let Ok(stripped) = path.strip_prefix(parent)
        // {
        //   path = stripped;
        // };

        // format!(
        //   "[{}:{}]",
        //   name,
        //   path.to_string_lossy(),
        // )
        name
      },
      ModuleParent::Module(parent) => {
        let parent_desc = self.describe_module(*parent);
        format!("{parent_desc}::{name}")
      },
    }
  }

  fn get_root_module(&self, mut module: C::ModuleReference) -> C::ModuleReference {
    // traverse parents until we get the root module with a
    // PathBuf
    loop {
      match &self.rget(module).parent {
        ModuleParent::Path { .. } => break,
        &ModuleParent::Module(next_id) => module = next_id,
      };
    };

    // store and mark the file handle as read
    let ModuleParent::Path(_) = &self.rget(module).parent else {
      unreachable!();
    };

    module
  }

  fn get_path(&self, module: C::ModuleReference) -> &ModulePath<C> {
    let root = self.get_root_module(module);
    let root = self.rget(root);

    // store and mark the file handle as read
    let ModuleParent::Path(path) = &root.parent else {
      unreachable!();
    };

    path
  }
}

pub trait Compiler: Debug + Sized + Clone + Copy + PartialEq + Eq
  where for<'a> Self::Store<'a>: CompilerPoolStore<Self>
{
  type Store<'a>;

  type ModuleReference: CompilerReference + Hash;
  type FunctionReference: CompilerReference + Hash;

  type TokensReference: CompilerReference;

  type OverwriteTypeReference: Debug + Clone;
}
