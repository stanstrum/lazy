mod r#type;
mod function;
mod expression;

use crate::{impl_ast, Result};
use crate::compiler::{
  Compiler,
  CompilerWorkflow,
};

use crate::asterizer::{
  Ast,
  Asterizer,
  ast::*,
  error::*,
};
use crate::tokenizer::{
  TokenKind,
  Punctuation,
  SpanStart,
};

impl<W: CompilerWorkflow> TopLevelNamespace<W> {
  /// Make an empty TopLevelNamespace in the case of an empty module
  pub(in crate::pipeline::asterizer) fn new_empty(start: SpanStart<W>) -> Self {
    Self {
      children: vec![],
      span: start.into_span(0),
    }
  }
}

impl_ast!(Identifier: (_, aster, start) => {
  let Some(TokenKind::Identifier(name)) = aster.reader.next_kind() else {
    return Ok(None);
  };

  Ok(Some(Self {
    name: name.into(),
    span: aster.finish_span(start),
  }))
});

impl_ast!(Qualified: (compiler, aster, start) => {
  let mut parts = vec![];
  let implicit;

  // If the qualified identifier begins with a double colon, then it's an
  // implicit identifier (details are otherwise resolved using context)
  if let Some(TokenKind::Punctuation(Punctuation::DoubleColon)) = aster.reader.peek_kind() {
    // Consume the double colon
    aster.reader.seek();

    implicit = true;
  } else {
    // Otherwise, this is a definite qualified identifier -- a.k.a. a non-
    // implicit one
    implicit = false;
  };

  loop {
    // Push mark in case we're doing reading
    aster.reader.push_mark();
    // Skip leading whitespace
    aster.reader.seek_whitespace_and_comments();

    // Check if there's another part to read
    let Some(part) = aster.make(compiler)? else {
      // If not, pop the mark and break out
      aster.reader.pop_mark();
      break;
    };

    // Otherwise, drop the mark, push the child, and continue onto the next part
    aster.reader.drop_mark();
    parts.push(part);
  };

  Ok(Some(Self {
    implicit,
    parts,
    span: aster.finish_span(start),
  }))
});

impl_ast!(Namespace: @stub);

impl_ast!(NamespaceChild: (compiler, aster, _) => {
  #[allow(clippy::manual_map)]
  Ok({
    if let Some(namespace) = aster.make(compiler)? {
      Some(Self::Namespace(Box::new(namespace)))
    } else if let Some(function) = aster.make(compiler)? {
      Some(Self::Function(function))
    } else {
      None
    }
  })
});

impl_ast!(TopLevelNamespace: (compiler, aster, start) => {
  let mut children = vec![];

  while {
    // Skip whitespace and comments
    aster.reader.seek_whitespace_and_comments();
    // While there are non-whitespace/comment Tokens left
    !aster.reader.is_empty()
  } {
    let Some(child) = aster.make(compiler)? else {
      return ExpectedSnafu {
        what: What::TopLevelNamespace,
        span: aster.next_read_span(compiler)?,
      }.fail()?;
    };

    children.push(child);
    aster.reader.seek_whitespace_and_comments();

    let Some(TokenKind::Punctuation(Punctuation::Semicolon)) = aster.reader.next_kind() else {
      return ExpectedSnafu {
        what: What::Semicolon,
        span: aster.next_read_span(compiler)?,
      }.fail()?;
    };
  };

  Ok(Some(Self {
    children,
    span: aster.finish_span(start),
  }))
});
