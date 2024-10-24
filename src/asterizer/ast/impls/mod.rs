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
  errors::*,
};
use crate::tokenizer::{
  SpanStart,
  TokenKind,
};

impl<W: CompilerWorkflow> TopLevelNamespace<W> {
  /// Make an empty TopLevelNamespace in the case of an empty module
  pub(in crate::asterizer) fn new_empty(start: SpanStart<W>) -> Self {
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

  while !aster.reader.is_empty() {
    let Some(child) = aster.make(compiler)? else {
      return ExpectedSnafu { what: What::TopLevelNamespace }.fail()?;
    };

    children.push(child);

    aster.reader.seek_whitespace_and_comments();
  };

  Ok(Some(Self {
    children,
    span: aster.finish_span(start),
  }))
});
