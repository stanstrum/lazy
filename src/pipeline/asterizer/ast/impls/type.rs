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

impl_ast!(Type: (compiler, aster, _) => {
  #[allow(clippy::manual_map)]
  Ok({
    if let Some(identifier) = aster.make(compiler)? {
      Some(Self::Qualified(identifier))
    } else {
      None
    }
  })
});
