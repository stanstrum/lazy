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

impl<W: CompilerWorkflow> Ast<W> for Type<W> {
  fn make(compiler: &mut Compiler<W>, aster: &mut Asterizer<W>, _start: SpanStart<W>) -> Result<Option<Self>> {
    #[allow(clippy::manual_map)]
    Ok({
      if let Some(identifier) = aster.make(compiler)? {
        Some(Self::Identifier(identifier))
      } else {
        None
      }
    })
  }
}
