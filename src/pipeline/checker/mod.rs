mod impls;
mod modifications;
pub(crate) mod error;

use crate::compiler::workflow::DefaultWorkflow;
use crate::{Result, ok};

use std::collections::VecDeque;
use std::rc::Rc;

use crate::compiler::{
  Check,
  CompilationStage,
  Compiler,
  CompilerJob,
  CompilerModule,
  CompilerModulePath,
  CompilerStoreHandle,
  CompilerWorkflow,
};

use modifications::*;
use crate::translator::lang::*;
use error::*;

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

fn get_main_handle(compiler: &mut Compiler<DefaultWorkflow>) -> Result<CompilerStoreHandle<DefaultWorkflow>> {
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
    Self {
      handle,
      input,
    }
  }

  fn check(self, compiler: &mut Compiler<DefaultWorkflow>) -> Result<Self::Out> {
    trace!("{:#?}", &self.input);

    if matches!(&compiler.store.get_module(&self.handle).path, CompilerModulePath::Real(_)) {
      let std_handle = get_main_handle(compiler)?;
      let CompilerJob::Checked(std) = &compiler.store.get_module(&std_handle).data else {
        unreachable!();
      };

      {
        let mut this = self.input.borrow_mut();

        for export in std.borrow().exports.iter() {
          this.imports.push(Import(export.get_reference().clone()));
        };
      };
    };

    let mut counter = 1;
    loop {
      trace!("check: resolve pass #{counter}");

      // Make new queue of modifications for this pass to add to
      let mut mods = Modifications::new();

      // Do resolution work and add modifications to `mods` -- chaining along
      // errors if there are any
      self.input.borrow().resolve(&mut mods)?;

      // If no modifications to the program structure are suggested, then just
      // break out of the loop
      if mods.is_empty() {
        trace!("check: resolve pass #{counter}: complete; no modifications found");

        break;
      };

      // Otherwise, apply those modifications
      mods.apply_all()?;

      // Since there were modifications found, we aren't done resolving types
      // and comparing them, so continue to the next iteration
      counter += 1;
    };

    // At this point, the checker isn't able to resolve the program contents
    // any further.  We'll do one last pass through the program hierarchy to
    // detect any unresolved bits, at which point we will throw an error.
    // Otherwise, this code is ready to be generated
    self.input.borrow().ensure_resolved(compiler)?;

    Ok(self.input)
  }
}
