use crate::{impl_ast, Result};
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

impl_ast!(Binding: @todo);

impl<W: CompilerWorkflow> Ast<W> for BlockExpression<W> {
  fn make(_compiler: &mut Compiler<W>, _aster: &mut Asterizer<W>, _start: SpanStart<W>) -> Result<Option<Self>> {
    todo!()
  }
}
