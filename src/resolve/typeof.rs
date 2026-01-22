use std::collections::VecDeque;

use crate::lang::{self, Lazy};
use crate::resolve::task::Task;
use crate::resolve::reference::TypeReference;

use super::Error;

pub fn type_of(lazy: &Lazy, reference: TypeReference, tasks: &mut VecDeque<Task>) -> Result<Option<lang::ty::Type>, Error> {
  todo!()
}
