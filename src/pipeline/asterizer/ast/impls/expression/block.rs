use crate::asterizer::{ast::*, error::*, Ast, Asterizer};
use crate::compiler::Compiler;
use crate::tokenizer::{Grouping, Punctuation, SpanStart, TokenKind};
use crate::{impl_ast, Result};

impl_ast!(Binding: @stub);

impl_ast!(BlockChild: (compiler, aster, _) => {
  #[allow(clippy::manual_map)]
  Ok({
    if let Some(expression) = aster.make(compiler)? {
      Some(Self::Expression(expression))
    } else if let Some(binding) = aster.make(compiler)? {
      Some(Self::Binding(binding))
    } else {
      None
    }
  })
});

impl_ast!(BlockExpression: (compiler, aster, start) => {
  // A block expression always starts with an opening curly brace.  Having this
  // check here prevents infinite recursion ... sometimes
  let Some(TokenKind::Grouping(Grouping::OpenBrace)) = aster.reader.next_kind() else {
    return Ok(None);
  };

  let mut children = vec![];
  let mut return_last = None;

  loop {
    // Skip whitespace and comments before each statement
    aster.reader.seek_whitespace_and_comments();

    // A statement can begin with an ending closing brace to denote an empty
    // function block
    if let Some(TokenKind::Grouping(Grouping::CloseBrace)) = aster.reader.peek_kind() {
      // Consume the peek
      aster.reader.seek();

      // Break out of loop -- we're done
      break;
    };

    // If there was no closing brace, then we're looking for a block child
    let Some(child) = aster.make(compiler)? else {
      // Invalid otherwise
      return ExpectedSnafu {
        what: What::Expression,
        span: aster.next_read_span(compiler)?,
      }.fail()?;
    };

    // Skip whitespace and comments following our statement
    aster.reader.seek_whitespace_and_comments();

    // Check if there's a semicolon hereafter
    if let Some(TokenKind::Punctuation(Punctuation::Semicolon)) = aster.reader.peek_kind() {
      // If so, consume it
      aster.reader.seek();

      // Add `child` to the list of children, since we're not treating it as a
      // return-last statement
      children.push(child);

      // Since we found a semicolon, we're going to do another iteration
      // starting after the semicolon
      continue;
    };

    // Otherwise, no semicolon means that this is a return-last statement.
    // Unfortunately, we read for a block child earlier, which means we
    // possibly have a Binding statement at the moment.  Handle `child`
    // appropriately:
    match child {
      BlockChild::Binding(binding) => {
        // Push the binding as a new statement and just have the return last
        // empty.  It's a simpler approach than forbidding this kind of syntax.
        children.push(BlockChild::Binding(binding));
      },
      BlockChild::Expression(expression) => {
        // Otherwise, appropriately place the parsed expression into
        // `return_last`
        return_last = Some(expression);
      },
    };

    // Now, we can't let the beginning of the loop close us out because that
    // could possibly parse another, unexpected expression where we expect
    // the end of the statement immediately
    let Some(TokenKind::Grouping(Grouping::CloseBrace)) = aster.reader.next_kind() else {
      // Invalid otherwise
      return ExpectedSnafu {
        what: What::ClosingBrace,
        span: aster.next_read_span(compiler)?,
      }.fail()?;
    };

    // Now everything is taken care of.  Break out and return.
    break;
  };

  Ok(Some(Self {
    children,
    return_last,
    span: aster.finish_span(start),
  }))
});
