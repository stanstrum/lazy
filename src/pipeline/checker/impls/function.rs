use super::*;
use crate::translator::ScopeParent;

impl Resolve for FunctionArgument {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    self.ty.resolve(mods)?;

    ok
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    self.ty.ensure_resolved(compiler)
  }
}

impl Resolve for LiteralInstruction {
  fn resolve(&self, _mods: &mut Modifications) -> Result {
    warn!("{}: LiteralInstruction::resolve", enchant!("stub"));
    ok
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    self.ty.ensure_resolved(compiler)
  }
}

impl Resolve for BlockInstruction {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    for instruction in self.instructions.iter() {
      instruction.resolve(mods)?;
    }
    ok
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    for instruction in self.instructions.iter() {
      instruction.ensure_resolved(compiler)?;
    }
    ok
  }
}

impl<C: CoerceWith<T>, T> CoerceWith<RcCell<T>> for C {
  fn coerce_with(&self, with: &RcCell<T>, mods: &mut Modifications) -> Result {
    self.coerce_with(&*with.borrow(), mods)
  }
}

impl CoerceWith<Type<Module>> for LiteralInstruction {
  fn coerce_with(&self, with: &Type<Module>, mods: &mut Modifications) -> Result {
    self.ty.coerce_with(with, mods)
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
        if a != b {
          panic!("coerce_with failed");
        };
      },
      (Type::UnresolvedInstrinsic { weak, .. }, other) => {
        match weak.upgrade().unwrap().as_ref() {
          LiteralInstructionKind::Integer(value) => {
            let Some(mut with) = with.make_wholly_unique() else {
              warn!("{}: couldn't resolve an intrinsic because the base isn't resolved yet", enchant!("coerce_with"));
              return ok;
            };

            let mut union = Type::new_union([
              Type::new_intrinsic(Intrinsic::I8, self.scope_parent().unwrap()),
              Type::new_intrinsic(Intrinsic::I16, self.scope_parent().unwrap()),
              Type::new_intrinsic(Intrinsic::I32, self.scope_parent().unwrap()),
              Type::new_intrinsic(Intrinsic::I64, self.scope_parent().unwrap()),
              Type::new_intrinsic(Intrinsic::U8, self.scope_parent().unwrap()),
              Type::new_intrinsic(Intrinsic::U16, self.scope_parent().unwrap()),
              Type::new_intrinsic(Intrinsic::U32, self.scope_parent().unwrap()),
              Type::new_intrinsic(Intrinsic::U64, self.scope_parent().unwrap()),
              Type::new_intrinsic(Intrinsic::F32, self.scope_parent().unwrap()),
              Type::new_intrinsic(Intrinsic::F64, self.scope_parent().unwrap()),
            ]);
            with.coerce_with_mut(&mut union)?;

            mods.push(Type::make_coerce_type(self, with));
          },
          LiteralInstructionKind::Float(value) => todo!(),
          LiteralInstructionKind::String(value) => todo!(),
        };
      },
      other => todo!("{other:#?}"),
    };
    ok
  }
}

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
            warn!("{}: union part doesn't extend and won't be used to coerce", enchant!("coerce_with_mut"));
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

impl CoerceWith<RcCell<Type<Module>>> for BlockInstruction {
  fn coerce_with(&self, with: &RcCell<Type<Module>>, mods: &mut Modifications) -> Result {
    for instruction in self.instructions.iter() {
      if let Instruction::Return { value, .. } = &*instruction.borrow() {
        if let Some(value) = value {
          value.coerce_with(with, mods)?;
        } else {
          let parent = self
            .parent().unwrap().upgrade().unwrap()
            .scope_parent().unwrap().upgrade().unwrap()
            .scope_parent().unwrap();

          Type::Intrinsic {
            kind: Intrinsic::Void,
            parent: parent.into(),
          }.coerce(with, mods)?;
        };
      };
    };
    ok
  }
}

impl CoerceWith<RcCell<Type<Module>>> for RcCell<Instruction> {
  fn coerce_with(&self, with: &RcCell<Type<Module>>, mods: &mut Modifications) -> Result {
    match &*self.borrow() {
      Instruction::Literal(literal_instruction) => literal_instruction.coerce_with(with, mods),
      Instruction::Block(block_instruction) => block_instruction.coerce_with(with, mods),
      Instruction::Return { parent, value } => todo!(),
    }
  }
}

impl TypeOf for Instruction {
  fn type_of(&self) -> Type<Module> {
    todo!()
  }
}

impl Resolve for Instruction {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    match self {
      Instruction::Literal(literal_instruction) => literal_instruction.resolve(mods),
      Instruction::Block(block_instruction) => block_instruction.resolve(mods),
      Instruction::Return { value, parent } => {
        let parent = parent.as_ref()
          .upgrade()
          .as_ref()
          .and_then(ScopeParent::scope_parent)
          .as_ref()
          .and_then(Weak::upgrade)
          .unwrap();
        let module = parent.scope_parent().unwrap();
        let return_type = parent.borrow().return_ty.clone();

        if let Some(value) = value {
          value.resolve(mods)?;
          return_type.coerce(value, mods)?;
        } else {
          return_type.coerce_with(
            &Type::Intrinsic {
              kind: Intrinsic::Void,
              parent: module.into(),
            },
            mods,
          )?;
        };
        ok
      },
    }
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    match self {
      Instruction::Literal(literal_instruction) => literal_instruction.ensure_resolved(compiler),
      Instruction::Block(block_instruction) => block_instruction.ensure_resolved(compiler),
      Instruction::Return { value: Some(value), .. } => value.ensure_resolved(compiler),
      Instruction::Return { value: None, .. } => ok,
    }
  }
}

impl Resolve for FunctionBlock {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    for instruction in self.children.iter() {
      instruction.resolve(mods)?;
    }
    ok
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    for instruction in self.children.iter() {
      instruction.ensure_resolved(compiler)?;
    }
    ok
  }
}

impl Resolve for Function {
  fn resolve(&self, mods: &mut Modifications) -> Result {
    for argument in self.arguments.iter() {
      argument.resolve(mods)?;
    }

    self.return_ty.resolve(mods)?;
    self.body.resolve(mods)?;

    ok
  }

  fn ensure_resolved(&self, compiler: &Compiler<DefaultWorkflow>) -> Result {
    for argument in self.arguments.iter() {
      argument.ensure_resolved(compiler)?;
    }

    self.return_ty.ensure_resolved(compiler)?;
    self.body.ensure_resolved(compiler)?;

    ok
  }
}
