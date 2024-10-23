mod r#type;
mod function;
mod expression;

use crate::Result;
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

impl TopLevelNamespace {
  /// Make an empty TopLevelNamespace in the case of an empty module
  pub(in crate::asterizer) fn new_empty(start: SpanStart) -> Self {
    Self {
      children: vec![],
      span: start.into_span(0),
    }
  }
}

impl<W: CompilerWorkflow> Ast<W> for Identifier {
  fn make(_compiler: &mut Compiler<W>, aster: &mut Asterizer<W>, start: SpanStart) -> Result<Option<Self>> {
    let Some(TokenKind::Identifier(name)) = aster.reader.next_kind() else {
      return Ok(None);
    };

    Ok(Some(Self {
      name: name.into(),
      span: aster.finish_span(start),
    }))
  }
}

impl<W: CompilerWorkflow> Ast<W> for Namespace {
  fn make(_: &mut Compiler<W>, _: &mut Asterizer<W>, _: SpanStart) -> Result<Option<Self>> {
    trace!("Namespace Ast::make stub");

    Ok(None)
  }
}

impl<W: CompilerWorkflow> Ast<W> for NamespaceChild {
  fn make(compiler: &mut Compiler<W>, aster: &mut Asterizer<W>, _: SpanStart) -> Result<Option<Self>> {
    Ok({
      if let Some(namespace) = aster.make(compiler)? {
        Some(Self::Namespace(Box::new(namespace)))
      } else if let Some(function) = aster.make(compiler)? {
        Some(Self::Function(function))
      } else {
        None
      }
    })
  }
}

impl<W: CompilerWorkflow> Ast<W> for TopLevelNamespace {
  fn make(compiler: &mut Compiler<W>, aster: &mut Asterizer<W>, start: SpanStart) -> Result<Option<Self>> {
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
  }
}
