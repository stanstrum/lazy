mod typing;
mod tasks;

use std::cell::RefCell;
use std::collections::VecDeque;

use pprint::Pretty;
use tasks::{Task, TaskResponse};
use lang::ty::{Type, TypeOf, TypeValue};
use lang::{Compiler, CompilerPoolStore};
use lazy_macros::{print_message, line_dbg};

use lang::reference::Reference;

pub(crate) type Result<C, T = ()> = std::result::Result<T, Box<ResolveError<C>>>;

pub use lang::error::ResolveError;
pub use lang::error::ResolveErrorBase;

use crate::typing::{Coerce, Resolve};

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

  pub fn push(&mut self, task: impl Task<C> + 'static, _source: &'static str) {
    // #[cfg(debug_assertions)] println!(line_dbg!("push from {}"), _source);
    self.tasks.borrow_mut().push_back(Box::new(task));
  }
}

impl<'store, 'pool, 'tasks, C: Compiler> Resolver<'store, 'pool, 'tasks, C> {
  fn work<T>(&mut self, description: String, cb: impl FnOnce(&mut Self) -> T) -> T {
    #[cfg(debug_assertions)]
    // Push the description to the stack
    self.tasks.trace.borrow_mut().push(description);

    // #[cfg(debug_assertions)]
    // // SPONGE: Print the explain() message for the whole stack
    // println!("{}", self.explain(0));

    // Run the task
    let result = cb(self);

    #[cfg(debug_assertions)]
    // Drop the handle
    self.tasks.trace.borrow_mut().pop()
      .expect("to pop status from trace");

    // Return the result, error or not
    result
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

impl<C: Compiler> Resolve<C> for Type<C> {
  fn resolve(&self, resolver: &Resolver<C>) -> Result<C, bool> {
    let ty = if let Some(ty) = &self.ty {
      ty
    } else {
      let Some(ty) = &self.reference.rget_from(resolver.store).ty else {
        return Ok(false);
      };

      ty
    };

    match &self.reference {
      lang::reference::TypeReference::Alias(alias_reference) => todo!(),
      lang::reference::TypeReference::Part(type_part_reference) => todo!(),
      lang::reference::TypeReference::Expression(expression_reference) => todo!(),
      lang::reference::TypeReference::Block(block_reference) => todo!(),
      lang::reference::TypeReference::ReturnTypeOf(function_reference) => {
        let block_reference = function_reference.rget_from(resolver.store).body;
        let block_ty_reference = lang::reference::TypeReference::Block(block_reference);
        let block_ty: Type<C> = block_ty_reference.into();

        block_ty.coerce(resolver, &self.reference)?;
      },
      lang::reference::TypeReference::Variable(variable_reference) => todo!(),
      lang::reference::TypeReference::StructMember(struct_reference, _) => todo!(),
    };

    match ty {
      TypeValue::Reference(type_reference) => todo!(),
      TypeValue::Resolved { part, span } => todo!(),
      TypeValue::Unresolved { module, qualified } => Ok(false),
      TypeValue::Intrinsic { kind, span } => Ok(true),
      TypeValue::WeakInteger { span } => Ok(false),
      TypeValue::WeakFloat { span } => Ok(false),
      TypeValue::WeakString { kind, characters, span, dereferenced } => Ok(false),
      TypeValue::Weak { span } => Ok(false),
      TypeValue::ReferenceTo { ty, r#mut, span } => todo!(),
      TypeValue::UnsizedArrayOf { ty, span } => todo!(),
      TypeValue::SizedArrayOf { ty, size, span } => todo!(),
      TypeValue::Struct { prototype } => todo!(),
    }
  }
}

pub fn resolve_and_verify<C: Compiler + 'static>(store: &mut C::Store<'_>, global: C::ModuleReference) -> Result<C> {
  let tasks = &mut Tasks::new();
  let mut resolver = Resolver::new(store, global, tasks)?;

  let _std = resolver.store.get_std()
    .expect("failed to load standard library");

  let main = find_main(&resolver, global)?;

  let mut types = typing::function::typify_function_reference(&resolver, main).collect::<VecDeque<_>>();

  for pass in 1.. {
    if types.is_empty() {
      break;
    };

    println!(line_dbg!("Resolve pass {}"), pass);

    for i in 0..types.len() {
      let ty = types.pop_front().unwrap();

      println!(line_dbg!("{}: {}"), i, ty.print(resolver.store));

      let is_resolved = Type::from(ty).resolve(&mut resolver)?;

      if !is_resolved {
        types.push_back(ty);
      };
    };

    resolver.resolve_tasks(line_dbg!("Finish pass").into())?;
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
