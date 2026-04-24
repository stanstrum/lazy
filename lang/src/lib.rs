pub mod span;
pub mod token;
pub mod intrinsic;

pub mod module;
pub mod function;
pub mod import;

pub mod ty;
pub mod reference;

pub mod expr;

use std::{fmt::Debug, hash::Hash};

use string_pool::StringPool;

use crate::module::{ModuleParent, ModulePath};

pub trait CompilerReference: Debug + Clone + Copy + PartialEq + Eq {}
impl<T: Debug + Clone + Copy + PartialEq + Eq> CompilerReference for T {}

pub trait CompilerPoolStore<C: Compiler>:
  reference::Store<C::ModuleReference, Out = module::Module<C>> +
  reference::Store<C::FunctionReference, Out = function::Function<C>> +
  reference::Store<C::TokensReference, Out = C::Tokens>
{
  fn pool(&self) -> &StringPool;

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

pub trait Compiler: Debug + Sized
  where for<'a> Self::Store<'a>: CompilerPoolStore<Self>
{
  type Store<'a>;

  type ModuleReference: CompilerReference + Hash;
  type FunctionReference: CompilerReference + Hash;

  type Tokens: Debug;
  type TokensReference: CompilerReference;

  type OverwriteTypeReference: Debug + Clone;
}
