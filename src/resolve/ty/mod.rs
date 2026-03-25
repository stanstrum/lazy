use crate::aster::pprint::Pretty;
use crate::lang::ty::Type;
use crate::lang::Lazy;
use crate::lang::reference::{Reference, TypePartReference, TypeReference, VariableReference};
use crate::line_dbg;
use crate::resolve::coerce::{SpecialPair, TypePair};
use crate::resolve::task_work;
use crate::resolve::tasks::{OverwriteType, ResolveAsTask, Subjugate};
use crate::resolve::ty::unknown::resolve_qualified_to_type;

use super::{Result, Error, Tasks, Resolve};

mod unknown;

impl Resolve for TypePartReference {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    // let description = format!(line_dbg!("Resolve TypePartReference: {}"), self.print(lazy));

    // task_work(tasks, description, |tasks| {
    let reference = TypeReference::Part(*self);
    let ty = self.rget_from(lazy);

    SpecialPair(&reference, ty).resolve(lazy, tasks)
    // })
  }
}

impl Resolve for TypeReference {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    let description = format!(line_dbg!("Resolve TypeReference: {}"), self.print(lazy));

    task_work(tasks, description, |tasks| match self {
      TypeReference::Part(type_part_reference) => {
        let ty = type_part_reference.rget_from(lazy);

        SpecialPair(self, ty).resolve(lazy, tasks)
      },
      TypeReference::ReturnTypeOf(function_reference) => {
        let ty = &function_reference.rget_from(lazy).header.ret_ty;

        SpecialPair(self, ty).resolve(lazy, tasks)
      },
      TypeReference::Alias(alias) => {
        alias.resolve(lazy, tasks)
      },
      TypeReference::Variable(v) => {
        let variable = v.rget_from(lazy);
        let ty = &variable.ty;

        SpecialPair(self, ty).resolve(lazy, tasks)
      },
      TypeReference::Expression(_) => todo!(),
      TypeReference::Block(_) => todo!(),
    })
  }
}

impl<'a, 'b> Resolve for TypePair<'a, 'b> {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()> {
    let SpecialPair(reference, ty) = self;

    let description = format!(line_dbg!("Resolve TypePair:\n- Ref.: {}\n- Type: {}"),
      reference.print(lazy),
      ty.print(lazy),
    );

    task_work(tasks, description, |tasks| {
      match ty {
        Type::Unresolved { module, qualified } => {
          if let Some(ty) = resolve_qualified_to_type(lazy, *module, qualified)? {
            tasks.push(Subjugate {
              after: Box::new(ResolveAsTask::<TypeReference> {
                reference: **reference,
              }),
              prerequisite: Box::new(OverwriteType {
                dest: **reference,
                src: ty,
              }),
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
        | Type::SizedArrayOf { ty, .. }
        | Type::Resolved { part: ty, .. }
          => ty.resolve(lazy, tasks),
        // Type::Expression(expression_reference) => todo!(),
        | Type::Reference(reference) => {
          let ty = reference.rget_from(lazy);
          SpecialPair(reference, ty).resolve(lazy, tasks)
        },
      }
    })
  }
}
