pub mod span;
pub mod token;
pub mod intrinsic;

pub mod module;
pub mod function;
pub mod import;

pub mod expr;

pub mod ty;
pub mod reference;

mod get_span;
mod type_of;

mod store;
pub mod keys;
pub mod tasks;
pub mod error;

use std::{fmt::Debug, hash::Hash, path::{Path, PathBuf}};

use string_pool::StringPool;

use crate::{error::LazyError, function::FunctionHeader, module::{ModuleParent, ModulePath}, reference::{BlockReference, TypePartReference, TypeReference}, ty::{Qualified, QualifiedSearchSpace}};

pub trait CompilerReference: Debug + Clone + Copy + PartialEq + Eq {}
impl<T: Debug + Clone + Copy + PartialEq + Eq> CompilerReference for T {}

pub trait CompilerPoolStore<'pool, C: Compiler>:
  reference::Store<C::ModuleReference, Out = module::Module<C>> +
  reference::Store<C::FunctionReference, Out = function::Function<C>> +
  reference::Store<C::TokensReference, Out = token::Tokens<C>> +
  reference::Store<BlockReference<C>, Out = expr::BlockExpression<C>> +
  reference::Store<TypeReference<C>, Out = ty::TypeKind<C>> +
  reference::Store<TypePartReference<C>, Out = ty::TypeKind<C>> +
{
  type Error: Debug;

  fn pool(&self) -> &'pool StringPool;
  fn pool_keys(&self) -> &crate::keys::PoolKeys;

  /// Sounds like a rough time.
  ///
  /// Returns a [`ModuleReference`] to the standard library, tokenizing those
  /// structures if necessary
  fn get_std(&mut self) -> Result<C::ModuleReference, LazyError<C>>;

  fn unwrap_std(&self) -> C::ModuleReference;

  /// Creates a module with the provided values.  This module's
  /// [`ModuleParent`] will be [`ModuleParent::Path`] (from `path`) and this
  /// path will be resolved either using the provided path in `relative_to`, or
  /// the current working directory using [`std::env::current_dir`].
  ///
  /// The path will be validated and then the source code will be parsed for
  /// tokens and AST.  If successful, the corresponding [`ModuleReference`] will
  /// be returned.
  fn add_file(&mut self, name: &str, path: PathBuf, relative_to: Option<&Path>) -> Result<C::ModuleReference, LazyError<C>>;

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

  fn create_module(&mut self, name: &str, parent: impl FnOnce(C::TokensReference, C::ModuleReference) -> ModuleParent<C>) -> C::ModuleReference;
}

pub trait Compiler: Debug + Sized + Clone + Copy + PartialEq + Eq {
  type Store<'a>: CompilerPoolStore<'a, Self>;

  type ModuleReference: CompilerReference + Hash;
  type FunctionReference: CompilerReference + Hash;

  type TokensReference: CompilerReference;

  fn resolve_qualified_to_space<'a>(
    store: &mut Self::Store<'a>,
    module: Self::ModuleReference,
    qualified: &Qualified<Self>,
    option: &Option<&mut tasks::Tasks<Self>>,
  ) -> Result<Option<QualifiedSearchSpace<Self>>, Box<error::ResolveError<Self>>>;
}
