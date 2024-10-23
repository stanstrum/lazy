use crate::Result;
use crate::compiler::{
  Compiler,
  CompilerWorkflow,
};

use crate::tokenizer::{
  Punctuation,
  SpanStart,
  TokenKind,
};
use crate::asterizer::{
  Ast,
  Asterizer,
  ast::*,
  errors::*,
};

impl<W: CompilerWorkflow> Ast<W> for BlockExpression {
  fn make(compiler: &mut Compiler<W>, aster: &mut Asterizer<W>, start: SpanStart) -> Result<Option<Self>> {
    todo!()
  }
}
