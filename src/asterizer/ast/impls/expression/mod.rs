use crate::Result;
use crate::compiler::{
  Compiler,
  CompilerWorkflow,
};

use crate::tokenizer::SpanStart;
use crate::asterizer::{
  Ast,
  Asterizer,
  ast::*,
};

impl<W: CompilerWorkflow> Ast<W> for BlockExpression {
  fn make(_compiler: &mut Compiler<W>, _aster: &mut Asterizer<W>, _start: SpanStart) -> Result<Option<Self>> {
    todo!()
  }
}
