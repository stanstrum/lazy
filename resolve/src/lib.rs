pub mod impls;
pub mod tasks;

use lang::{Compiler, CompilerPoolStore, ty::TypeOf};
use lazy_macros::{print_message, line_dbg};

use pprint::Pretty;
use lang::intrinsic::Intrinsic;
use lang::span::GetSpan;
use lang::ty::TypeKind;
use lang::reference::{Reference, Store, TypeReference};

use tasks::Tasks;

pub(crate) type Result<C, T = ()> = std::result::Result<T, Box<ResolveError<C>>>;

pub use lang::error::ResolveError;
pub use lang::error::ResolveErrorBase;

struct Resolver<'store, 'pool, C: Compiler> {
  store: &'store mut C::Store<'pool>,
  tasks: Tasks<C>,
  global: C::ModuleReference,
  std: C::ModuleReference,
}

pub trait Resolve<C: Compiler> {
  fn resolve(&self, store: &C::Store<'_>, tasks: &mut Tasks<C>) -> Result<C>;
}

pub trait Coerce<C: Compiler> {
  fn coerce(&self, store: &C::Store<'_>, other: &impl TypeOf<C>, tasks: &mut Tasks<C>) -> Result<C>;
}

// impl<R: Copy> TypeOf for R
//   where for<'a> store<'a>: Store<R>,
//         for<'a> <store<'a> as Store<R>>::Out: TypeOf
// {
//   fn type_of(&self, store: &store) -> Result<Option<Type>> {
//     self.rget_from(store).type_of(store)
//   }

//   fn reference(&self, store: &store) -> Option<TypeReference> {
//     self.rget_from(store).reference(store)
//   }
// }

impl<'store, 'pool, C: Compiler + 'static> Resolver<'store, 'pool, C> {
  fn new(store: &'store mut C::Store<'pool>, global: C::ModuleReference) -> Result<C, Self> {
    let mut tasks = Tasks::new();

    let std = tasks.work::<Result<C, C::ModuleReference>>(
      line_dbg!("Get standard library").into(),
      |tasks| match store.get_std() {
        Ok(std) => Ok(std),
        Err(err) => tasks.seed_error(ResolveErrorBase::Lazy(Box::new(err))),
      },
    )?;

    let this = Self {
      store,
      tasks,
      global,
      std,
    };

    Ok(this)
  }

  fn resolve_tasks(&mut self, description: String) -> Result<C> {
    self.tasks.work(description, |tasks| loop {
      // Resolve `global` recursively
      impls::structure::resolve_module_reference(self.store, &self.global, tasks)?;

      // Execute the tasks: typically overwriting unknown values with &mut
      let did_execute = tasks.execute_pass(self.store)?;

      // If no tasks ran, we _should_ be finished resolving
      if !did_execute {
        return Ok(());
      };
    })
  }
}

fn find_main<C: Compiler + 'static>(store: &C::Store<'_>, module: C::ModuleReference, tasks: &mut Tasks<C>) -> Result<C, C::FunctionReference> {
  let main_search = {
    let main_id = store.pool().insert("main");

    module.rget_from(store)
      .functions.iter()
      .find(|&function| {
        function.rget_from(store)
          .header.name.id == main_id
      })
  };

  let Some(main) = main_search else {
    let root = store.get_root_module(module);
    let module_name = store.describe_module(root);

    return tasks.seed_error(ResolveErrorBase::MissingEntryPoint {
      module_name,
      file: module,
    });
  };

  {
    let module_name = store.describe_module(module);
    let function = main.rget_from(store);
    let span = function.header.name.span;

    print_message!(store, {
      level: Debug,
      force: false,
      description: format!(line_dbg!("{} has the entry point \"main\""), module_name),
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

pub fn resolve_and_verify<C: Compiler + 'static>(store: &mut C::Store<'_>, global: C::ModuleReference) -> Result<C> {
  let mut resolver = Resolver::new(store, global)?;

  resolver.resolve_tasks(line_dbg!("Resolve global").into())?;

  resolver.tasks.work::<Result<C>>(
    line_dbg!("Make default ambiguous types").into(),
    |tasks| {
      impls::structure::default_types_in_module(resolver.store, &global, tasks)?;

      Ok(())
    },
  )?;

  resolver.resolve_tasks(line_dbg!("Resolve after make default ambiguous types").into())?;

  resolver.tasks.work::<Result<C>>(
    line_dbg!("Verify global").into(),
    |tasks| {
      impls::structure::verify_module(resolver.store, &global, tasks)?;

      print_message!(resolver.store, {
        level: Stub,
        force: false,
        description: line_dbg!("verify rest of program, apart from main").into(),
        contents: MessageContents::File::<C>(global),
      });

      Ok(())
    },
  )?;

  resolver.tasks.work::<Result<C>>(
    line_dbg!("Verify main").into(),
    |tasks| {
      // get main and error if it's not present
      let main = find_main(resolver.store, global, tasks)?;

      let borrow = (&*resolver.store).rget(main);
      let ret_ty_reference = TypeReference::ReturnTypeOf(main);

      // set up some perfunctory data to coerce return type to i32
      // TODO: eventually just coerce main as fn(...) -> ...
      {
        let ret_ty = &borrow.header.ret_ty;
        let span = ret_ty.get_span(resolver.store);

        print_message!(resolver.store, {
          level: Debug,
          force: false,
          description: format!(line_dbg!("{reference} is {ty}"),
            reference = ret_ty_reference.print(resolver.store),
            ty = ret_ty.print(resolver.store),
          ),
          contents: MessageContents::WithinSource(WithinSource::new(vec![
            MessageSection {
              text: "here".into(),
              span,
            }
          ])),
        });

        ret_ty_reference.coerce(resolver.store, &TypeKind::Intrinsic {
          kind: Intrinsic::I32,
          span,
        }, tasks)?;
      };

      Ok(())
    },
  )?;

  print_message!(resolver.store, {
    level: Info,
    force: false,
    description: line_dbg!("No further work should be done.").into(),
    contents: MessageContents::None::<C>,
  });
  assert!(
    !resolver.tasks.execute_pass(store)?,
    "verifying should not have queued any more work",
  );

  Ok(())
}
