use std::collections::VecDeque;
use std::rc::Rc;

use crate::{Result, ok};

use crate::compiler::{
  Compiler,
  CompilerStoreHandle,
  CompilerWorkflow,
  Check,
};

use crate::translator::lang::*;

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
}

#[allow(unused)]
#[derive(Debug)]
enum Modification {
  ResolveUnresolvedTypeModuleReference {
    weak: WeakCell<UnresolvedReference<Module>>,
    value: Reference<Type<Module>, Module>,
  },
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
            weak: Rc::downgrade(&rc),
            value: Self::Resolved(found.upgrade().unwrap()),
          });
        };
      },
    }; ok
  }
}

impl Resolve for Type<Module> {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    match self {
      Type::Intrinsic { .. } => ok,
      Type::Reference(reference) => reference.resolve(mods),
    }
  }
}

impl Resolve for FunctionArgument {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    self.ty.borrow().resolve(mods)?;

    ok
  }
}

impl Resolve for Function {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    for argument in self.arguments.iter() {
      argument.borrow().resolve(mods)?;
    };

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
}

impl Resolve for Module {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    for child in self.children.iter() {
      child.borrow().resolve(mods)?;
    };

    ok
  }
}

impl<W: CompilerWorkflow> Check<W> for Checker<W> {
  type In = RcCell<Module>;
  type Out = RcCell<Module>;

  fn new(input: Self::In, handle: CompilerStoreHandle<W>) -> Self {
    Self {
      handle,
      input,
    }
  }

  fn check(self, _compiler: &mut Compiler<W>) -> Result<Self::Out> {
    trace!("{:#?}", &self.input);

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

    Ok(self.input)
  }
}
