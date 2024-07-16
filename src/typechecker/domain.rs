use std::path::PathBuf;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use crate::compiler::{
  DebugInfo,
  Handle,
};

use crate::typechecker::lang;

#[allow(unused)]
#[derive(Debug)]
pub(super) struct NamedDomainMember {
  pub(super) name: String,
  pub(super) member: DomainMember,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) enum DomainMemberKind {
  Domain(Domain),
  Function(lang::Function),
  ExternFunction(lang::ExternFunction),
  Type(lang::TypeCell),
  Struct(lang::Struct),
}

#[derive(Debug)]
pub(crate) struct DomainMember {
  pub(crate) kind: DomainMemberKind,
  pub(crate) template_scope: Option<Rc<RefCell<Vec<(String, lang::TypeCell)>>>>,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Domain {
  pub(crate) inner: HashMap<String, DomainMember>,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub(crate) struct DomainReference {
  pub(crate) handle: Handle,
  pub(crate) inner: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct ProgramData {
  pub(crate) domain: Domain,
  pub(crate) debug_info: DebugInfo,
  pub(crate) path: PathBuf,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Program {
  pub(crate) inner: HashMap<Handle, ProgramData>,
}

impl DomainReference {
  pub(super) fn new(handle: Handle) -> Self {
    Self {
      inner: vec![],
      handle,
    }
  }
}

impl Program {
  pub(crate) fn new() -> Self {
    Self {
      inner: HashMap::new(),
    }
  }

  pub(crate) fn from_map(inner: HashMap<Handle, ProgramData>) -> Self {
    Self { inner }
  }
}
