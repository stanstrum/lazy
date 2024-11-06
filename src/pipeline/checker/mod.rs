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

use crate::translator::lang::*;
use error::*;

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Modifications {
  modifications: VecDeque<Modification>,
}

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

#[allow(unused)]
#[derive(Debug)]
enum Modification {
  ResolveUnresolvedTypeModuleReference {
    weak: WeakCell<UnresolvedReference<Module>>,
    value: Reference<Type<Module>, Module>,
  },
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

impl Modification {
  fn apply(self) -> Result {
    todo!()
  }
}

impl Modifications {
  fn new() -> Self {
    Self {
      modifications: VecDeque::new(),
    }
  }

  fn push(&mut self, modification: Modification) {
    self.modifications.push_back(modification);
  }

  fn is_empty(&self) -> bool {
    self.modifications.is_empty()
  }

  fn apply_all(self) -> Result {
    for (i, modification) in (1..).zip(self.modifications) {
      trace!("check: modification #{i}");
      modification.apply()?;
    };

    ok
  }
}

impl Resolve for Reference<Type<Module>, Module> {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    match self {
      Reference::Resolved(_) => {},
      Reference::Unresolved(rc) => {
        let search = rc.borrow().find_reference()?;

        if let ScopeSearch::Found(found) = search {
          mods.push(Modification::ResolveUnresolvedTypeModuleReference {
            weak: Rc::downgrade(rc),
            value: Self::Resolved(found.upgrade().unwrap()),
          });
        };
      },
    }; ok
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    match self {
      Reference::Resolved(rc) => rc.borrow().ensure_resolved(compiler),
      Reference::Unresolved(rc) => {
        let span = compiler.span_to_read_span(rc.borrow().span)?;

        UnresolvedQualifiedSnafu { span }.fail()?
      },
    }
  }
}

impl Resolve for Type<Module> {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    match self {
      Type::Intrinsic { .. } => ok,
      Type::Reference(reference) => reference.resolve(mods),
    }
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    match self {
      Type::Intrinsic { .. } => ok,
      Type::Reference(reference) => reference.ensure_resolved(compiler),
    }
  }
}

impl Resolve for FunctionArgument {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    self.ty.borrow().resolve(mods)?;

    ok
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    self.ty.borrow().ensure_resolved(compiler)
  }
}

impl Resolve for Function {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    for argument in self.arguments.iter() {
      argument.borrow().resolve(mods)?;
    };

    self.return_ty.borrow().resolve(mods)?;

    ok
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    for argument in self.arguments.iter() {
      argument.borrow().ensure_resolved(compiler)?;
    };

    self.return_ty.borrow().ensure_resolved(compiler)?;

    ok
  }
}

impl Resolve for ModuleChild {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    match self {
      ModuleChild::Function(rc) => rc.borrow().resolve(mods),
      ModuleChild::Module(rc) => rc.borrow().resolve(mods),
    }
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    match self {
      ModuleChild::Function(rc) => rc.borrow().ensure_resolved(compiler),
      ModuleChild::Module(rc) => rc.borrow().ensure_resolved(compiler),
    }
  }
}

impl Resolve for Module {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    for child in self.children.iter() {
      child.borrow().resolve(mods)?;
    };

    ok
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    for child in self.children.iter() {
      child.borrow().ensure_resolved(compiler)?;
    };

    ok
  }
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
