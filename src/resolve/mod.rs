use crate::error::{Level, MessageContents, PrintableMessage, print_message};
use crate::lang::ty::{Intrinsic, Qualified, Type};
use crate::line_dbg;
use crate::resolve::coerce::{Coerce, SpecialPair};
use crate::resolve::tasks::{ResolveType, Tasks};
use crate::lang::reference::{AliasReference, ExpressionReference, FunctionReference, ModuleReference, Reference, Store, TypePartReference, TypeReference};
use crate::lang::Lazy;
use crate::resolve::type_of::TypeOf;

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
    let reference = TypeReference::Part(*self);
    let ty = self.rget_from(lazy);

    ResolvedTypePair(&reference, ty).resolve(lazy, tasks)
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

  for (count, part) in qualified.parts.iter().enumerate() {
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
    QualifiedSearchSpace::Module(_) => todo!(),
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
      TypeReference::ArgumentOf(function_reference, index) => {
        let function = function_reference.rget_from(lazy);
        let variable = function.header.arguments.get(*index).unwrap();
        let ty = &variable.ty;

        ResolvedTypePair(self, ty).resolve(lazy, tasks)
      },
      TypeReference::Expression(_) => todo!(),
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
      Type::Intrinsic { .. } => {
        // do nothing ...
        Ok(())
      },
      Type::WeakInteger { .. } => todo!(),
      Type::WeakFloat { .. } => todo!(),
      Type::WeakString { .. } => todo!(),
      Type::Weak { .. } => todo!(),
      | Type::ReferenceTo { ty, .. }
      | Type::UnsizedArrayOf { ty, .. }
      | Type::SizedArrayOf { ty, .. } => {
        ty.resolve(lazy, tasks)
      },
      // Type::Expression(expression_reference) => todo!(),
      Type::Reference(reference) => {
        let ty = reference.rget_from(lazy);
        ResolvedTypePair(reference, ty).resolve(lazy, tasks)
      },
    }
  }
}

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

    print_message(lazy, PrintableMessage {
      level: Level::Debug,
      force: false,
      description: line_dbg!("stub").into(),
      contents: MessageContents::File(function.parent),
    });

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

pub fn task_resolve(lazy: &mut Lazy, module: ModuleReference) -> Result<()> {
  let mut tasks = Tasks::new();

  tasks.task_status(line_dbg!("resolve global").into());

  loop {
    module.resolve(lazy, &mut tasks)?;
    let did_execute = tasks.execute_pass(lazy)?;

    if !did_execute {
      break;
    };
  };

  Ok(())
}
