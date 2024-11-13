pub(crate) mod error;
mod impls;
mod modifications;

use std::collections::VecDeque;
use std::rc::Rc;

use error::*;
pub(crate) use impls::*;
use modifications::*;

use crate::compiler::workflow::DefaultWorkflow;
use crate::compiler::{
  Check, CompilationStage, Compiler, CompilerJob, CompilerModule, CompilerModulePath,
  CompilerStoreHandle, CompilerWorkflow,
};
use crate::translator::lang::*;
use crate::{enchant, ok, Result};

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Checker<W: CompilerWorkflow> {
  input: RcCell<Module>,
  handle: CompilerStoreHandle<W>,
}

trait Resolve: Sized {
  fn resolve(&self, mods: &mut Modifications) -> Result;
  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result;
}

trait TypeOf {
  fn type_of(&self) -> Type<Module>;
}

trait Coerce<C: CoerceWith<T>, T> {
  fn coerce(&self, what: &C, mods: &mut Modifications) -> Result;
}

trait CoerceWith<T> {
  fn coerce_with(&self, with: &T, mods: &mut Modifications) -> Result;
}

impl<C: CoerceWith<T>, T> Coerce<C, T> for T {
  fn coerce(&self, what: &C, mods: &mut Modifications) -> Result {
    what.coerce_with(self, mods)
  }
}

impl<T: Resolve> Resolve for RcCell<T> {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    self.borrow().resolve(mods)
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    self.borrow().ensure_resolved(compiler)
  }
}

fn get_main_handle(
  compiler: &mut Compiler<DefaultWorkflow>,
) -> Result<CompilerStoreHandle<DefaultWorkflow>> {
  let handle = compiler.store.register_module(&CompilerModule {
    path: CompilerModulePath::ImplicitSource {
      name: "index.zy",
      content: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/std/index.zy")),
    },
    data: CompilerJob::Taken,
  });

  if let CompilerJob::Taken = compiler.store.get_module(&handle).data {
    compiler.store.get_module_mut(&handle).data = CompilerJob::Unprocessed;
  };

  compiler.store.get_module(&handle).data.stage();

  compiler.bring_to_stage(&handle, CompilationStage::Generate)?;

  Ok(handle)
}

impl Check<DefaultWorkflow> for Checker<DefaultWorkflow> {
  type In = RcCell<Module>;
  type Out = RcCell<Module>;

  fn new(input: Self::In, handle: CompilerStoreHandle<DefaultWorkflow>) -> Self {
    Self { handle, input }
  }

  fn check(self, compiler: &mut Compiler<DefaultWorkflow>) -> Result<Self::Out> {
    if matches!(
      &compiler.store.get_module(&self.handle).path,
      CompilerModulePath::Real(_)
    ) {
      let std_handle = get_main_handle(compiler)?;
      let CompilerJob::Checked(std) = &compiler.store.get_module(&std_handle).data else {
        unreachable!();
      };

      {
        let mut this = self.input.borrow_mut();

        for export in std.borrow().exports.iter() {
          let Some(name) = export.get_name(&*compiler)? else {
            warn!(
              "{}: couldn't resolve reference, therefore couldn't resolve name",
              enchant!("")
            );
            continue;
          };

          this.imports.push(Import {
            name,
            reference: export.get_reference().upgrade().unwrap(),
          });
        }
      };
    };

    let name = enchant!("check");

    let mut counter = 1;
    loop {
      // Make new queue of modifications for this pass to add to
      let mut mods = Modifications::new();

      // Do resolution work and add modifications to `mods` -- chaining along
      // errors if there are any
      self.input.resolve(&mut mods)?;

      // If no modifications to the program structure are suggested, then just
      // break out of the loop
      if mods.is_empty() {
        trace!("{name}: resolve pass #{counter}: complete; no modifications found");

        break;
      };

      // Otherwise, apply those modifications
      mods.apply_all()?;

      // Since there were modifications found, we aren't done resolving types
      // and comparing them, so continue to the next iteration
      counter += 1;
    }

    // At this point, the checker isn't able to resolve the program contents
    // any further.  We'll do one last pass through the program hierarchy to
    // detect any unresolved bits, at which point we will throw an error.
    // Otherwise, this code is ready to be generated
    self.input.ensure_resolved(compiler)?;

    Ok(self.input)
  }
}
