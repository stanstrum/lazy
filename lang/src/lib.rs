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
mod get_span;

use std::{fmt::Debug, hash::Hash, path::{Path, PathBuf}};

use string_pool::StringPool;

use crate::{function::FunctionHeader, module::{ModuleParent, ModulePath}, reference::{BlockReference, TypePartReference, TypeReference}, ty::OverwriteTypeReference};

pub trait CompilerReference: Debug + Clone + Copy + PartialEq + Eq {}
impl<T: Debug + Clone + Copy + PartialEq + Eq> CompilerReference for T {}

pub trait CompilerPoolStore<'a, C: Compiler>:
  reference::Store<C::ModuleReference, Out = module::Module<C>> +
  reference::Store<C::FunctionReference, Out = function::Function<C>> +
  reference::Store<C::TokensReference, Out = token::Tokens<C>> +
  reference::Store<BlockReference<C>, Out = expr::BlockExpression<C>> +
  reference::Store<TypeReference<C>, Out = ty::Type<C>> +
  reference::Store<TypePartReference<C>, Out = ty::Type<C>> +
  reference::Store<OverwriteTypeReference<C>, Out = ty::Type<C>> +
  // reference::Store<TypePartReference<C>, Out = ty::Type<C>> +
{
  type Error;

  fn pool(&self) -> &'a StringPool;

  /// Creates a module with the provided values.  This module's
  /// [`ModuleParent`] will be [`ModuleParent::Path`] (from `path`) and this
  /// path will be resolved either using the provided path in `relative_to`, or
  /// the current working directory using [`std::env::current_dir`].
  ///
  /// The path will be validated and then the source code will be parsed for
  /// tokens and AST.  If successful, the corresponding [`ModuleReference`] will
  /// be returned.
  fn add_file(&mut self, name: &str, path: PathBuf, relative_to: Option<&Path>) -> Result<C::ModuleReference, Self::Error>;

  fn create_function(&mut self, module: C::ModuleReference, header: FunctionHeader<C>) -> C::FunctionReference;

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
  where for<'a> Self::Store<'a>: CompilerPoolStore<'a, Self>
{
  type Store<'a>;

  type ModuleReference: CompilerReference + Hash;
  type FunctionReference: CompilerReference + Hash;

  type TokensReference: CompilerReference;
}
