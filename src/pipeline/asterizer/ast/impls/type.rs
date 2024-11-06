use super::*;

use impls::{Keyword, TokenKind};

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

impl_ast!(TypeAlias: (compiler, aster, start) => {
  let Some(TokenKind::Keyword(Keyword::Type)) = aster.reader.next_kind() else {
    return Ok(None);
  };

  aster.reader.seek_whitespace_and_comments();

  let Some(name) = aster.make(compiler)? else {
    return ExpectedSnafu {
      what: What::Identifier,
      span: aster.next_read_span(compiler)?,
    }.fail()?;
  };

  aster.reader.seek_whitespace_and_comments();

  let Some(TokenKind::Punctuation(Punctuation::Bollocks)) = aster.reader.next_kind() else {
    return ExpectedSnafu {
      what: What::Punctuation,
      span: aster.next_read_span(compiler)?
    }.fail()?;
  };

  aster.reader.seek_whitespace_and_comments();

  let Some(ty) = aster.make(compiler)? else {
    return ExpectedSnafu {
      what: What::Type,
      span: aster.next_read_span(compiler)?,
    }.fail()?;
  };

  Ok(Some(Self {
    name,
    ty,
    span: aster.finish_span(start),
  }))
});

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
