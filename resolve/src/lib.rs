mod typing;
mod tasks;
mod resolver;

use std::collections::VecDeque;

use lazy_macros::{print_message, line_dbg};
use lang::ty::{Type, TypeOf, TypeValue};
use lang::reference::Reference;
use lang::{Compiler, CompilerPoolStore};
use tasks::{Tasks, Task, TaskResponse};

use resolver::Resolver;

pub use lang::error::ResolveError;
pub use lang::error::ResolveErrorBase;

pub(crate) type Result<C, T = ()> = std::result::Result<T, Box<ResolveError<C>>>;

fn find_main<C: Compiler + 'static>(resolver: &Resolver<C>, module: C::ModuleReference) -> Result<C, C::FunctionReference> {
  let main_search = {
    let main_id = resolver.store.pool().insert("main");

    module.rget_from(resolver.store)
      .functions.iter()
      .find(|&function| {
        function.rget_from(resolver.store)
          .header.name.id == main_id
      })
  };

  let Some(main) = main_search else {
    let root = resolver.store.get_root_module(module);
    let module_name = resolver.store.describe_module(root);

    return resolver.seed_error(ResolveErrorBase::MissingEntryPoint {
      module_name,
      file: module,
    });
  };

  {
    let module_name = resolver.store.describe_module(module);
    let function = main.rget_from(resolver.store);
    let span = function.header.name.span;

    print_message!(resolver.store, {
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
  let tasks = &mut Tasks::new();
  let mut resolver = Resolver::new(store, global, tasks)?;

  let _std = resolver.store.get_std()
    .expect("failed to load standard library");

  let main = find_main(&resolver, global)?;

  let mut types = typing::function::typify_function_reference(&resolver, main)
    .collect::<VecDeque<_>>();

  for pass in 1.. {
    if types.is_empty() {
      break;
    };

    println!(line_dbg!("Resolve pass {}"), pass);

    let mut have_resolved = false;
    for i in 0..types.len() {
      let dyn_obj = types.pop_front().unwrap();

      // println!(line_dbg!("{}: {}"), i, dyn_obj.print(resolver.store));

      let is_resolved = dyn_obj.resolve(&mut resolver)?;

      if !is_resolved {
        types.push_back(dyn_obj);
      } else {
        have_resolved = true;
      };
    };

    resolver.resolve_tasks(format!(line_dbg!("Finish pass {}"), pass))?;

    assert!(have_resolved, "nothing happened!");
  };

  todo!();

  resolver.resolve_tasks(line_dbg!("Resolve global").into())?;

  resolver.work::<Result<C>>(
    line_dbg!("Make default ambiguous types").into(),
    |resolver| {
      todo!();
      // impls::structure::default_types_in_module(resolver.store, &global, tasks)?;

      Ok(())
    },
  )?;

  // resolver.resolve_tasks(line_dbg!("Resolve after make default ambiguous types").into())?;

  // resolver.tasks.work::<Result<C>>(
  //   line_dbg!("Verify global").into(),
  //   |tasks| {
  //     impls::structure::verify_module(resolver.store, &global, tasks)?;

  //     print_message!(resolver.store, {
  //       level: Stub,
  //       force: false,
  //       description: line_dbg!("verify rest of program, apart from main").into(),
  //       contents: MessageContents::File::<C>(global),
  //     });

  //     Ok(())
  //   },
  // )?;

  // resolver.tasks.work::<Result<C>>(
  //   line_dbg!("Verify main").into(),
  //   |tasks| {
  //     // get main and error if it's not present
  //     let main = find_main(resolver.store, global, tasks)?;

  //     let borrow = (&*resolver.store).rget(main);
  //     let ret_ty_reference = TypeReference::ReturnTypeOf(main);

  //     // set up some perfunctory data to coerce return type to i32
  //     // TODO: eventually just coerce main as fn(...) -> ...
  //     {
  //       let ret_ty = &borrow.header.ret_ty;
  //       let span = ret_ty.get_span(resolver.store);

  //       print_message!(resolver.store, {
  //         level: Debug,
  //         force: false,
  //         description: format!(line_dbg!("{reference} is {ty}"),
  //           reference = ret_ty_reference.print(resolver.store),
  //           ty = ret_ty.print(resolver.store),
  //         ),
  //         contents: MessageContents::WithinSource(WithinSource::new(vec![
  //           MessageSection {
  //             text: "here".into(),
  //             span,
  //           }
  //         ])),
  //       });

  //       ret_ty_reference.coerce(resolver.store, &TypeValue::Intrinsic {
  //         kind: Intrinsic::I32,
  //         span,
  //       }, tasks)?;
  //     };

  //     Ok(())
  //   },
  // )?;

  // print_message!(resolver.store, {
  //   level: Info,
  //   force: false,
  //   description: line_dbg!("No further work should be done.").into(),
  //   contents: MessageContents::None::<C>,
  // });
  // assert!(
  //   !resolver.tasks.execute_pass(store)?,
  //   "verifying should not have queued any more work",
  // );

  todo!();
  Ok(())
}
