use crate::Result;
use crate::compiler::{
  Compiler,
  CompilerWorkflow,
};

use crate::asterizer::{
  Ast,
  Asterizer,
  ast::TopLevelNamespace,
};
use crate::tokenizer::{
  Span,
  SpanStart,
};

impl TopLevelNamespace {
  /// Make an empty TopLevelNamespace in the case of an empty module
  pub(in crate::asterizer) fn new_empty(start: SpanStart) -> Self {
    Self {
      children: vec![],
      span: start.into_span(0),
    }
  }
}

impl<W: CompilerWorkflow> Ast<W> for TopLevelNamespace {
  fn make(compiler: &mut Compiler<W>, aster: &mut Asterizer<W>, start: SpanStart) -> Result<Option<Self>> {
    todo!()
  }

  fn get_span(&self) -> Span {
    self.span
  }
}
