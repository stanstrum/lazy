use super::*;

#[allow(unused)]
#[derive(Debug)]
pub(crate) enum Modification {
  ResolveUnresolvedTypeModuleReference {
    weak: WeakCell<Reference<Type<Module>, Module>>,
    value: Reference<Type<Module>, Module>,
  },
  CoerceType {
    reference: RcCell<Type<Module>>,
    value: Type<Module>,
  },
}

#[allow(unused)]
#[derive(Debug)]
pub(super) struct Modifications {
  modifications: VecDeque<Modification>,
}

impl Modification {
  fn apply(self) -> Result {
    match self {
      Modification::ResolveUnresolvedTypeModuleReference { weak, value } => {
        *weak.upgrade().unwrap().borrow_mut() = value;
      },
      Modification::CoerceType { reference, value } => {
        *reference.borrow_mut() = value;
      },
    };

    ok
  }
}

impl Modifications {
  pub(super) fn new() -> Self {
    Self {
      modifications: VecDeque::new(),
    }
  }

  pub(super) fn push(&mut self, modification: Modification) {
    self.modifications.push_back(modification);
  }

  pub(super) fn is_empty(&self) -> bool {
    self.modifications.is_empty()
  }

  pub(super) fn apply_all(self) -> Result {
    for (i, modification) in (1..).zip(self.modifications) {
      trace!("{}: modification #{i}: {modification:#?}", crate::enchant!("check"));
      modification.apply()?;
    }

    ok
  }
}
