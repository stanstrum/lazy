use crate::{impl_ast, Result};
use crate::compiler::{
  Compiler,
  CompilerWorkflow,
};

use crate::tokenizer::{
  TokenKind,
  Grouping,
  SpanStart,
};
use crate::asterizer::{
  Ast,
  Asterizer,
  ast::*,
};

impl_ast!(Binding: @todo);

impl<W: CompilerWorkflow> Ast<W> for BlockChild<W> {
  fn make(compiler: &mut Compiler<W>, aster: &mut Asterizer<W>, _start: SpanStart<W>) -> Result<Option<Self>> {
    #[allow(clippy::manual_map)]
    Ok({
      if let Some(binding) = aster.make(compiler)? {
        Some(Self::Binding(binding))
      } else {
        None
      }
    })
  }
}

impl<W: CompilerWorkflow> Ast<W> for BlockExpression<W> {
  fn make(compiler: &mut Compiler<W>, aster: &mut Asterizer<W>, start: SpanStart<W>) -> Result<Option<Self>> {
    let Some(TokenKind::Grouping(Grouping::OpenBrace)) = aster.reader.next_kind() else {
      return Ok(None);
    };

    let mut children = vec![];

    loop {
      aster.reader.seek_whitespace_and_comments();

      let Some(child) = aster.make(compiler)? else {
        break;
      };

      children.push(child);
    };

    todo!();

    Ok(Some(Self {
      children,
      return_last: todo!(),
      span: aster.finish_span(start),
    }))
  }
}
