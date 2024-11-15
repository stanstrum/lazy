use super::*;

impl Extends<Type<Module>> for Type<Module> {
  fn extends(&self, other: &Type<Module>) -> bool {
    match (self, other) {
      (Type::Intrinsic { kind: a, .. }, Type::Intrinsic { kind: b, .. }) => a == b,
      (_, Type::Union(tys)) => {
        tys.borrow().iter().any(|ty| self.extends(ty))
      },
      other => {
        warn!("{}: stub extends: {self:#?} and {other:#?}", enchant!("extends"));
        false
      },
    }
  }
}

impl Type<Module> {
  fn coerce_with_mut(&mut self, with: &Type<Module>) -> Result {
    match (&self, with) {
      (_, Type::Union(tys)) => {
        for ty in tys.borrow().iter() {
          if self.extends(ty) {
            self.coerce_with_mut(ty)?;
          } else {
            // warn!("{}: union part doesn't extend and won't be used to coerce", enchant!("coerce_with_mut"));
          };
        };
      },
      (Type::Intrinsic { kind: a, .. }, Type::Intrinsic { kind: b, .. }) => {
        match (a, b) {
          _ if a == b => return ok,
          other => todo!("{other:#?}"),
        };
      },
      other => todo!("{other:#?}"),
    }; ok
  }
}

impl CoerceWith<Type<Module>> for RcCell<Type<Module>> {
  fn coerce_with(&self, with: &Type<Module>, mods: &mut Modifications) -> Result {
    match (&*self.borrow(), with) {
      (Type::Reference(reference), _) => {
        if let Some(reference) = reference.get_and_maybe_modify(mods)?.as_ref().and_then(Weak::upgrade) {
          reference.coerce_with(with, mods)?;
        } else {
          warn!("{}: couldn't coerce type because reference was unresolved", enchant!("coerce_with"));
        };
      },
      (_, Type::Reference(reference)) => {
        if let Some(reference) = reference.get_and_maybe_modify(mods)?.as_ref().and_then(Weak::upgrade) {
          reference.coerce(self, mods)?;
        } else {
          warn!("{}: couldn't coerce type because reference was unresolved", enchant!("coerce_with"));
        };
      },
      (Type::Intrinsic { kind: a, .. }, Type::Intrinsic { kind: b, .. }) => {
        if b == &Intrinsic::Unknown {
          return ok;
        };

        if a == &Intrinsic::Unknown {
          mods.push(Type::make_replace_type(self, with.clone()));
        } else if a != b {
          panic!("coerce_with failed");
        };
      },
      (Type::UnresolvedInstrinsic { weak, .. }, _) => {
        let kind = weak.upgrade().unwrap();
        match *kind {
          LiteralInstructionKind::Integer(_) => {
            let Some(mut with) = with.make_wholly_unique() else {
              warn!("{}: couldn't resolve an intrinsic because the base isn't resolved yet", enchant!("coerce_with"));
              return ok;
            };

            let mut union = kind.type_of(self.scope_parent().unwrap());
            with.coerce_with_mut(&mut union)?;

            mods.push(Type::make_replace_type(self, with));
          },
          LiteralInstructionKind::Float(_) => todo!(),
          LiteralInstructionKind::String(_) => todo!(),
        };
      },
      (_, Type::UnresolvedInstrinsic { .. }) => {
        warn!("{}: coerce_with: Type::UnresolvedIntrinsic", enchant!("stub"));
      },
      other => todo!("{other:#?}"),
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
