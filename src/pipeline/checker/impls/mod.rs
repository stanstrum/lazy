use std::rc::Weak;

use super::*;

mod function;
mod module;

pub(crate) use module::*;

impl<V: SearchIn<S>, S: Scope> Resolve for RcCell<Reference<V, S>> {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    self.get_and_maybe_modify(mods)?;

    ok
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    let this = self.borrow();

    let Reference::Resolved(_) = &*this else {
      let span = this
        .get_inner_weak()
        .as_ref()
        .and_then(Weak::upgrade)
        .unwrap()
        .borrow()
        .span(compiler);
      return UnresolvedQualifiedSnafu { span }.fail()?;
    };

    ok
  }
}

impl GetAndMaybeModify<Type<Module>, Module> for RcCell<Type<Module>> {
  fn get_and_maybe_modify(
    &self,
    _mods: &mut Modifications,
  ) -> Result<Option<WeakCell<Type<Module>>>> {
    todo!()
  }
}

impl Resolve for RcCell<Type<Module>> {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    match &*self.borrow() {
      Type::Intrinsic { .. } => ok,
      Type::Reference(reference) => reference.resolve(mods),
      other => todo!("{other:#?}"),
    }
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    match &*self.borrow() {
      Type::Intrinsic { kind, parent } if kind == &Intrinsic::Unknown => {
        let span = parent.as_ref().upgrade().unwrap().borrow().span;
        let span = compiler.span_to_read_span(span)?;
        UnresolvedLiteralSnafu { span }.fail()?
      },
      Type::Intrinsic { .. } => ok,
      Type::Reference(reference) => reference.ensure_resolved(compiler),
      Type::OfExpression { weak } => weak.upgrade().unwrap().ensure_resolved(compiler),
      Type::Union(_) => todo!(),
      Type::UnresolvedInstrinsic { span, .. } => {
        let span = compiler.span_to_read_span(*span)?;
        UnresolvedLiteralSnafu { span }.fail()?
      },
    }
  }
}
