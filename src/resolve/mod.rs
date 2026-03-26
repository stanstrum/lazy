mod impls;
mod tasks;

use crate::lang::reference::FunctionReference;
use crate::line_dbg;
use crate::error::*;

use crate::tokenize::token::Span;
use crate::lang::ty::Type;
use crate::lang::reference::{ModuleReference, Reference, Store, TypeReference};
use crate::lang::Lazy;

use tasks::Tasks;

type Result<T> = std::result::Result<T, Box<Error>>;

#[derive(Debug)]
pub enum Error {
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
pub struct SpecialPair<'a, S: Store<R>, R: Reference<S>>(
  pub &'a R,
  pub &'a S::Out,
);

pub type TypePair<'a, 'b> = SpecialPair<'a, Lazy<'b>, TypeReference>;

trait Resolve {
  fn resolve(&self, lazy: &Lazy, tasks: &mut Tasks) -> Result<()>;
}

pub trait Coerce {
  fn coerce(&self, lazy: &Lazy, other: &impl TypeOf, tasks: &mut Tasks) -> Result<()>;
}

pub trait TypeOf {
  fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>>;
}

impl<R: Copy> TypeOf for R
  where for<'a> Lazy<'a>: Store<R>,
        for<'a> <Lazy<'a> as Store<R>>::Out: TypeOf
{
  fn type_of(&self, lazy: &Lazy) -> Result<Option<Type>> {
    lazy.rget(*self).type_of(lazy)
  }
}

fn find_main(lazy: &Lazy, module: ModuleReference) -> Result<FunctionReference> {
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

    return Err(Box::new(Error::MissingEntryPoint {
      module_name,
      file: module,
    }));
  };

  {
    let module_name = lazy.describe_module(module);
    let function = main.rget_from(lazy);
    let span = function.header.name.span;

    print_message(lazy, PrintableMessage {
      level: Level::Debug,
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

  tasks.work::<Result<()>>(
    line_dbg!("Resolve global").into(),
    |tasks| loop {
      // Resolve `global` recursively
      module.resolve(lazy, tasks)?;

      // Execute the tasks: typically overwriting unknown values with &mut
      let did_execute = tasks.execute_pass(lazy)?;

      // If no tasks ran, we _should_ be finished resolving
      if !did_execute {
        return Ok(());
      };
    },
  )?;

  tasks.work::<Result<()>>(
    line_dbg!("Verify global").into(),
    |tasks| {
      let main = find_main(lazy, module)?;

      impls::function::verify_function(lazy, main, tasks)?;

      println!(line_dbg!("stub: verify rest of program, apart from main"));

      Ok(())
    },
  )?;

  println!(line_dbg!("No further work should be done."));
  assert!(!tasks.execute_pass(lazy)?);

  Ok(())
}
