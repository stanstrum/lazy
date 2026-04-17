mod impls;
pub mod tasks;

use crate::{print_message, line_dbg};

use crate::aster::pprint::Pretty;
use crate::lang::reference::{FunctionReference, Store};
use crate::lang::span::GetSpan;

use crate::resolve::tasks::OverwriteTypeReference;
use crate::tokenize::token::Span;
use crate::lang::Lazy;
use crate::lang::ty::{Intrinsic, Type};
use crate::lang::reference::{ModuleReference, Reference, TypeReference};

use tasks::Tasks;

type Result<T> = std::result::Result<T, Box<Error>>;

#[derive(Debug)]
pub enum ErrorBase {
  MissingEntryPoint {
    module_name: String,
    file: ModuleReference,
  },
  UnknownTypeName {
    module_name: String,
    span: Span,
  },
  TypeMismatch {
    whence: &'static str,
    a_print: String,
    a_span: Span,
    b_print: String,
    b_span: Span,
  },
  UnresolvedInVerify {
    what: String,
    span: Span,
  },
}

#[derive(Debug)]
pub struct Error {
  pub base: ErrorBase,
  pub call_stack: String,
}

#[derive(Debug, Clone, Copy)]
pub enum TypePairModifier {
  Dereference,
}

#[derive(Debug, Clone)]
pub struct TypePair {
  pub reference: TypeReference,
  pub ty: Type,
  pub modifiers: Vec<TypePairModifier>,
}

pub trait Resolve {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()>;
}

pub trait Coerce {
  fn coerce(&self, lazy: &Lazy, other: &impl TypeOf, tasks: &mut Tasks) -> Result<()>;
}

pub trait TypeOf {
  fn type_of(&self, lazy: &Lazy) -> Option<Type>;
  fn reference(&self, lazy: &Lazy) -> Option<OverwriteTypeReference>;
}

impl TypePair {
  pub fn new(reference: TypeReference, ty: Type) -> Self {
    Self {
      reference,
      ty,
      modifiers: vec![],
    }
  }
}

// impl<R: Copy> TypeOf for R
//   where for<'a> Lazy<'a>: Store<R>,
//         for<'a> <Lazy<'a> as Store<R>>::Out: TypeOf
// {
//   fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>> {
//     self.rget_from(lazy).type_of(lazy)
//   }

//   fn reference(&self, lazy: &Lazy) -> Option<TypeReference> {
//     self.rget_from(lazy).reference(lazy)
//   }
// }

fn find_main(lazy: &Lazy, module: ModuleReference, tasks: &mut Tasks) -> Result<FunctionReference> {
  let main_search = {
    let main_id = lazy.pool.insert("main");

    module.rget_from(lazy)
      .functions.iter()
      .find(|&function| {
        function.rget_from(lazy)
          .header.name.id == main_id
      })
  };

  let Some(main) = main_search else {
    let root = lazy.get_root_module(module);
    let module_name = lazy.describe_module(root);

    return tasks.seed_error(ErrorBase::MissingEntryPoint {
      module_name,
      file: module,
    });
  };

  {
    let module_name = lazy.describe_module(module);
    let function = main.rget_from(lazy);
    let span = function.header.name.span;

    print_message!(lazy, {
      level: Debug,
      force: false,
      description: format!(line_dbg!("{} has the entrypoint \"main\""), module_name),
      contents: MessageContents::WithinSource(vec![WithinSource {
        range: span,
        sections: vec![MessageSection {
          text: "here".into(),
          span,
        }],
      }]),
    });
  };

  Ok(*main)
}

pub fn resolve_and_verify(lazy: &mut Lazy, module: ModuleReference) -> Result<()> {
  let mut tasks = Tasks::new();

  let resolve_tasks = |description, lazy: &mut _, tasks: &mut Tasks| -> Result<()> {
    tasks.work::<Result<()>>(description, |tasks| loop {
      // Resolve `global` recursively
      module.resolve(lazy, tasks)?;

      // Execute the tasks: typically overwriting unknown values with &mut
      let did_execute = tasks.execute_pass(lazy)?;

      // If no tasks ran, we _should_ be finished resolving
      if !did_execute {
        return Ok(());
      };
    })
  };

  resolve_tasks(line_dbg!("Resolve global").into(), lazy, &mut tasks)?;

  tasks.work::<Result<()>>(
    line_dbg!("Make default ambiguous types").into(),
    |tasks| {

      impls::structure::default_types_in_module(lazy, &module, tasks)?;

      Ok(())
    },
  )?;

  resolve_tasks(
    line_dbg!("Resolve after make default ambiguous types").into(),
    lazy, &mut tasks
  )?;

  tasks.work::<Result<()>>(
    line_dbg!("Verify global").into(),
    |tasks| {
      impls::structure::verify_module(lazy, &module, tasks)?;

      print_message!(lazy, {
        level: Stub,
        force: false,
        description: line_dbg!("verify rest of program, apart from main").into(),
        contents: MessageContents::File(module),
      });

      Ok(())
    },
  )?;

  tasks.work::<Result<()>>(
    line_dbg!("Verify main").into(),
    |tasks| {
      // get main and error if it's not present
      let main = find_main(lazy, module, tasks)?;

      let borrow = lazy.rget(main);
      let ret_ty_reference = TypeReference::ReturnTypeOf(main);

      // set up some perfunctory data to coerce return type to i32
      // TODO: eventually just coerce main as fn(...) -> ...
      {
        let ret_ty = &borrow.header.ret_ty;
        let span = ret_ty.get_span(lazy);

        print_message!(lazy, {
          level: Debug,
          force: false,
          description: format!(line_dbg!("{reference} is {ty}"),
            reference = ret_ty_reference.print(lazy),
            ty = ret_ty.print(lazy),
          ),
          contents: MessageContents::WithinSource(WithinSource::new(vec![
            MessageSection {
              text: "here".into(),
              span,
            }
          ])),
        });

        ret_ty_reference.coerce(lazy, &Type::Intrinsic {
          kind: Intrinsic::I32,
          span,
        }, tasks)?;
      };

      Ok(())
    },
  )?;

  print_message!(lazy, {
    level: Info,
    force: false,
    description: line_dbg!("No further work should be done.").into(),
    contents: MessageContents::None,
  });
  assert!(
    !tasks.execute_pass(lazy)?,
    "verifying should not have queued any more work",
  );

  Ok(())
}
