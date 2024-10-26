use crate::{impl_ast, Result};
use crate::compiler::{
  Compiler,
  CompilerWorkflow,
};

use crate::tokenizer::{
  TokenKind,
  Punctuation,
  Grouping,
  SpanStart,
};
use crate::asterizer::{
  Ast,
  Asterizer,
  ast::*,
  errors::*,
};

impl_ast!(Binding: @stub);

impl_ast!(BlockChild: (compiler, aster, _) => {
  #[allow(clippy::manual_map)]
  Ok({
    if let Some(binding) = aster.make(compiler)? {
      Some(Self::Binding(binding))
    } else {
      None
    }
  })
});

impl_ast!(BlockExpression: (compiler, aster, start) => {
  let Some(TokenKind::Grouping(Grouping::OpenBrace)) = aster.reader.next_kind() else {
    return Ok(None);
  };

  let mut children = vec![];
  let mut return_last = None;

  loop {
    aster.reader.seek_whitespace_and_comments();

    // At the beginning of each segment, there may be a closing brace to make
    // an empty block
    if let Some(TokenKind::Grouping(Grouping::CloseBrace)) = aster.reader.next_kind() {
      break;
    };

    // Otherwise, read for an expression
    let Some(child) = aster.make(compiler)? else {
      // Invalid otherwise
      return ExpectedSnafu { what: What::Expression }.fail()?;
    };

    // Push mark in case there is no semicolon after this statement
    aster.reader.push_mark();
    aster.reader.seek_whitespace_and_comments();

    // If there is a semicolon ...
    if let Some(TokenKind::Punctuation(Punctuation::Semicolon)) = aster.reader.next_kind() {
      // Then this expression shouldn't be returned from the block:
      // - Add it to the list of statements
      children.push(child);

      // - Drop mark
      aster.reader.drop_mark();
      // - Go on to the read next expression
      break;
    }

    match child {
      // If it's a binding, just pretend there's a semicolon.  Better than
      // returning an error specifically for this
      BlockChild::Binding(binding) => {
        // Add the binding as a statement
        children.push(BlockChild::Binding(binding));
      },
      // Elsewise, this expression actually returns last
      BlockChild::Expression(expression) => {
        // Set return_last instead
        return_last = Some(expression);
      },
    };

    // Let the next iteration close us out
  };

  let Some(TokenKind::Grouping(Grouping::CloseBrace)) = aster.reader.next_kind() else {
    return ExpectedSnafu { what: What::CloseBrace }.fail()?;
  };

  Ok(Some(Self {
    children,
    return_last,
    span: aster.finish_span(start),
  }))
});
