mod tasks;

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use lang::expr::BlockExpression;
use tasks::{Task, TaskResponse};
use lang::{Compiler, CompilerPoolStore, ty::TypeOf};
use lazy_macros::{print_message, line_dbg};

use pprint::Pretty;
use lang::intrinsic::Intrinsic;
use lang::span::GetSpan;
use lang::ty::TypeValue;
use lang::reference::{BlockReference, ExpressionReference, Reference, Store, TypeReference, VariableReference};

trait Resolve<C: Compiler> {
  fn resolve(&self, resolver: &Resolver<C>) -> Result<C>;
}

trait Coerce<C: Compiler> {
  fn coerce(&self, resolver: &Resolver<C>, other: &impl TypeOf<C>) -> Result<C>;
}

pub(crate) type Result<C, T = ()> = std::result::Result<T, Box<ResolveError<C>>>;

pub use lang::error::ResolveError;
pub use lang::error::ResolveErrorBase;

pub struct Tasks<C: Compiler> {
  tasks: RefCell<VecDeque<Box<dyn Task<C>>>>,
  trace: RefCell<Vec<String>>,
}

pub struct Resolver<'store, 'pool, 'tasks, C: Compiler> {
  store: &'store mut C::Store<'pool>,
  tasks: &'tasks Tasks<C>,
  global: C::ModuleReference,
}

impl<C: Compiler> Tasks<C> {
  fn new() -> Self {
    Self {
      tasks: RefCell::new(VecDeque::new()),
      trace: RefCell::new(vec![]),
    }
  }
}

impl<'store, 'pool, 'tasks, C: Compiler> Resolver<'store, 'pool, 'tasks, C> {
  fn work<T>(&mut self, description: String, cb: impl FnOnce(&mut Self) -> T) -> T {
    todo!()
  }

  /// Returns a boolean corresponding to whether any tasks were executed
  fn execute_pass(&mut self) -> Result<C, bool> {
    // #[cfg(debug_assertions)]
    // println!(line_dbg!("execute_pass start"));

    let total = self.tasks.tasks.borrow().len();

    if total == 0 {
      return Ok(false);
    };

    let mut count = 0;

    loop {
      let Some(task) = ({
        let mut borrow = self.tasks.tasks.borrow_mut();
        let value = borrow.pop_front();
        drop(borrow);
        value
      }) else {
        break;
      };

      let description = {
        let explain = task.explain::<'tasks>(self);

        format!(
          "execute_pass: {count}/{total}:\n{explain}",
        ).to_owned()
      };

      let response = self.work(description, |resolver| task.execute(resolver))?;

      match response {
        TaskResponse::Pop => {
          // do nothing
        },
        TaskResponse::Replace(replace) => {
          self.tasks.tasks.borrow_mut().push_back(replace);
        },
      };

      count += 1;
    };

    Ok(true)
  }

  fn explain(&self, offset: usize) -> String {
    let mut out = String::new();

    for (count, explain) in self.tasks.trace.borrow().iter().enumerate() {
      let spaces = " ".repeat(offset) + &"|  ".repeat(count);

      for line in explain.split('\n') {
        out += &format!("{spaces}{line}\n");
      };
    };

    // out += "###";

    out
  }

  fn seed_error<T>(&self, base: ResolveErrorBase<C>) -> Result<C, T> {
    let call_stack = format!("Call Stack:\n{}", self.explain(4));

    Err(Box::new(ResolveError {
      base,
      call_stack,
    }))
  }
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

impl<'store, 'pool, 'tasks, C: Compiler + 'static> Resolver<'store, 'pool, 'tasks, C> {
  fn new(store: &'store mut C::Store<'pool>, global: C::ModuleReference, tasks: &'tasks Tasks<C>) -> Result<C, Self> {
    Ok(Self {
      store,
      tasks,
      global,
    })
  }

  fn resolve_tasks(&mut self, description: String) -> Result<C> {
    self.execute_pass().and(Ok(()))
  }
}

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

    todo!();
    // return tasks.seed_error(ResolveErrorBase::MissingEntryPoint {
    //   module_name,
    //   file: module,
    // });
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

trait Typify<C: Compiler>: Sized {
  fn get_type_iter<'store>(self, store: &'store C::Store<'_>) -> Box<dyn Iterator<Item = TypeReference<C>> + 'store>;
}

fn typify_module_reference<'store, C: Compiler>(resolver: &Resolver<'store, '_, '_, C>, module_reference: C::ModuleReference) -> Box<dyn Iterator<Item = TypeReference<C>> + 'store> {
  todo!()
}

impl<C: Compiler + 'static> Typify<C> for ExpressionReference<C> {
  fn get_type_iter<'store>(self, store: &'store C::Store<'_>) -> Box<dyn Iterator<Item = TypeReference<C>> + 'store> {
    let m: Box::<dyn Iterator<Item = TypeReference<C>>> = match store.rget(self) {
      lang::expr::Expression::Block(block_reference) => Box::new(block_reference.get_type_iter(store)),
      lang::expr::Expression::Literal { value, span, out } => Box::new(std::iter::once(out.reference)),
      lang::expr::Expression::Variable { reference, span } => todo!(),
      lang::expr::Expression::Unknown { qualified, out } => todo!(),
      lang::expr::Expression::Unary { expr, op, span, out } => todo!(),
      lang::expr::Expression::Binary { a, b, op, span, out } => todo!(),
      lang::expr::Expression::StructInitializer { ty, members, span } => todo!(),
    };

    todo!()
  }
}

impl<C: Compiler + 'static> Typify<C> for BlockReference<C> {
  fn get_type_iter<'store>(self, store: &'store C::Store<'_>) -> Box<dyn Iterator<Item = TypeReference<C>> + 'store> {
    Box::new(store.rget(self).children.iter().map(move |expr_id| {
      let expression_reference = ExpressionReference(self, *expr_id);
      expression_reference.get_type_iter(store)
    }).flatten())
  }
}

fn typify_function_reference<'store, C: Compiler + 'static>(resolver: &'store Resolver<'store, '_, '_, C>, function_reference: C::FunctionReference) -> Box<dyn Iterator<Item = TypeReference<C>> + 'store> {
  let borrow = function_reference.rget_from(resolver.store);
  let args = (0..borrow.header.arguments.len())
    .map(move |index| TypeReference::Variable(
      VariableReference::Argument(function_reference, index)
    )
  );
  let return_type = std::iter::once(TypeReference::ReturnTypeOf(function_reference));

  let exprs = borrow.body.get_type_iter(resolver.store);

  Box::new(return_type.chain(args).chain(exprs))
}

pub fn resolve_and_verify<C: Compiler + 'static>(store: &mut C::Store<'_>, global: C::ModuleReference) -> Result<C> {
  let tasks = &mut Tasks::new();
  let mut resolver = Resolver::new(store, global, tasks)?;

  let _std = resolver.store.get_std()
    .expect("failed to load standard library");

  let main = find_main(&resolver, global)?;

  for r#type in typify_function_reference(&resolver, main) {
    println!(line_dbg!("{}"), r#type.print(resolver.store));
  };

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
