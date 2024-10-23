use crate::Result;
use crate::compiler::{
  Compiler,
  CompilerWorkflow,
};

use crate::asterizer::{
  TokenReader,
  Ast,
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
  fn make(compiler: &mut Compiler<W>, reader: &mut TokenReader, start: SpanStart) -> Result<Option<Self>> {
    todo!()
  }

  fn get_span(&self) -> Span {
    self.span
  }
}
