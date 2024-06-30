use std::collections::HashMap;

use crate::compiler::Handle;
use super::lang;

#[allow(unused)]
#[derive(Debug)]
pub(super) struct NamedDomainMember {
  pub(super) name: String,
  pub(super) member: DomainMember,
}

#[allow(unused)]
#[derive(Debug)]
pub(super) enum DomainMember {
  Domain(Domain),
  Function(lang::Function),
  Type(lang::TypeCell),
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Domain {
  pub(super) inner: HashMap<String, DomainMember>,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub(crate) struct DomainReference {
  pub(crate) handle: Handle,
  pub(crate) inner: Vec<String>,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Program {
  pub(crate) inner: HashMap<Handle, Domain>,
}

impl DomainReference {
  pub(super) fn new(handle: Handle) -> Self {
    Self {
      inner: vec![],
      handle,
    }
  }

  // fn push(&mut self, part: String) {
  //   self.inner.push(part)
  // }

  // fn push_all<T: Iterator<Item = String>>(&mut self, parts: T) {
  //   for part in parts {
  //     self.push(part);
  //   };
  // }
}

impl Program {
  pub(crate) fn new() -> Self {
    Self {
      inner: HashMap::new(),
    }
  }
}

impl From<HashMap<Handle, Domain>> for Program {
  fn from(inner: HashMap<Handle, Domain>) -> Self {
    Self { inner }
  }
}
