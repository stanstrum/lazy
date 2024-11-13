mod parse;
mod search;

use super::*;
use crate::enchant;

impl<S: Scope<Index = str>> Part<S> for ast::Identifier<DefaultWorkflow> {
  fn part_to_index(&self) -> &S::Index {
    self.name.as_str()
  }
}

impl<S: Scope<Index = usize>> Part<S> for usize {
  fn part_to_index(&self) -> &S::Index {
    self
  }
}

impl Scope for Function {
  type Index = str;
  type Part = ast::Identifier<DefaultWorkflow>;
}

impl Scope for FunctionBlock {
  type Index = usize;
  type Part = usize;
}
