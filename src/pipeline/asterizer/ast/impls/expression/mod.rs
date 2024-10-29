mod literal;
mod block;

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

impl_ast!(Expression: (compiler, aster, _) => {
  #[allow(clippy::manual_map)]
  Ok({
    if let Some(literal) = aster.make(compiler)? {
      Some(Self::Literal(literal))
    } else if let Some(block) = aster.make(compiler)?.map(Box::new) {
      Some(Self::Block(block))
    } else {
      None
    }
  })
});
