use crate::aster::pprint::Pretty;
use crate::lang::function::Function;
use crate::lang::module::{Module, TypeAlias};
use crate::lang::ty::{Intrinsic, Qualified, Type};
use crate::resolve::tasks::{ResolveType, TaskResponse, Tasks};
use crate::lang::reference::{AliasReference, FunctionReference, ModuleReference, Reference, Store, TypePartReference, TypeReference};
use crate::lang::Lazy;

mod tasks;
pub mod type_of;
pub mod coerce;

#[derive(Debug)]
pub enum Error {

}

type Result<T> = std::result::Result<T, Box<Error>>;

trait Resolve {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()>;
}

impl Resolve for TypePartReference {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    todo!()
  }
}

fn resolve_qualified_to_type(lazy: &Lazy, module: ModuleReference, qualified: &Qualified) -> Result<Option<Type>> {
  enum QualifiedSearchSpace {
    Type(Type),
    Module(ModuleReference),
  }

  if qualified.implicit {
    // Implicits are coerced before they are resolved
    return Ok(None);
  };

  let mut space = QualifiedSearchSpace::Module(module);

  let mut iter = qualified.parts.iter().enumerate();
  while let Some((count, part)) = iter.next() {
    if count == 0 {
      let part_string = lazy.pool.get(part.id).collect::<String>();
      if let Some(kind) = Intrinsic::try_from_str(&part_string) {
        space = QualifiedSearchSpace::Type(
          Type::Intrinsic {
            kind,
            span: part.span,
          }
        );

        continue;
      };
    };

    match space {
      QualifiedSearchSpace::Type(ty) => todo!("match space: {ty:#?}"),
      QualifiedSearchSpace::Module(module) => {
        // Look for type aliases by this name
        if let Some(id) = module.rget_from(lazy)
          .aliases.iter()
          .position(|alias| part.id == alias.name.id)
        {
          let alias = AliasReference(module, id);
          space = QualifiedSearchSpace::Type(Type::Reference(TypeReference::Alias(alias)));

          continue;
        };
      },
    };

    todo!()
  };

  match space {
    QualifiedSearchSpace::Type(ty) => Ok(Some(ty)),
    QualifiedSearchSpace::Module(module_reference) => todo!(),
  }
}

#[derive(Debug)]
struct ResolvedTypePair<'a>(&'a TypeReference, &'a Type);

impl Resolve for TypeReference {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    match self {
      TypeReference::Part(type_part_reference) => {
        let ty = type_part_reference.rget_from(lazy);

        ResolvedTypePair(self, ty).resolve(lazy, tasks)
      },
      TypeReference::ReturnTypeOf(function_reference) => {
        let ty = &function_reference.rget_from(lazy).header.ret_ty;

        ResolvedTypePair(self, ty).resolve(lazy, tasks)
      },
      TypeReference::Alias(_) => todo!(),
    }
  }
}

impl<'a> Resolve for ResolvedTypePair<'a> {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    let ResolvedTypePair(reference, ty) = self;

    match ty {
      Type::Unresolved { module, qualified } => {
        if let Some(ty) = resolve_qualified_to_type(lazy, *module, qualified)? {
          tasks.push(ResolveType {
            dest: **reference,
            value: ty,
          });
        };

        Ok(())
      },
      Type::Intrinsic { kind, span } => todo!(),
      Type::WeakInteger { span } => todo!(),
      Type::WeakFloat { span } => todo!(),
      Type::WeakString { span } => todo!(),
      Type::ReferenceTo { ty, r#mut, span } => todo!(),
      Type::UnsizedArrayOf { ty, span } => todo!(),
      Type::SizedArrayOf { ty, size, span } => todo!(),
      Type::Expression(expression_reference) => todo!(),
      Type::Reference(_) => todo!(),
    }
  }
}

impl Resolve for TypeAlias {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    todo!()
  }
}

impl Resolve for FunctionReference {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    let function = self.rget_from(lazy);

    TypeReference::ReturnTypeOf(*self).resolve(lazy, tasks)?;

    todo!()
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

    for alias in module.aliases.iter() {
      alias.resolve(lazy, tasks)?;
    };

    Ok(())
  }
}

pub fn task_resolve(lazy: &mut Lazy, module: ModuleReference) -> Result<()> {
  let mut tasks = Tasks::new();

  loop {
    module.resolve(lazy, &mut tasks)?;
    let did_execute = tasks.execute_pass(lazy)?;

    if !did_execute {
      break;
    };
  };

  Ok(())
}
