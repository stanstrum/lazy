use crate::{impl_ast, Result};
use crate::compiler::{
  Compiler,
  CompilerWorkflow,
};

use crate::tokenizer::{
  Punctuation,
  SpanStart,
  TokenKind,
};
use crate::asterizer::{
  Ast,
  Asterizer,
  ast::*,
  errors::*,
};

impl_ast!(FunctionArgument: (compiler, aster, start) => {
  let Some(identifier) = aster.make(compiler)? else {
    return Ok(None);
  };

  aster.reader.seek_whitespace_and_comments();

  let Some(TokenKind::Punctuation(Punctuation::Colon)) = aster.reader.next_kind() else {
    // TODO: should this be an error?
    return Ok(None);
  };

  aster.reader.seek_whitespace_and_comments();

  let Some(ty) = aster.make(compiler)? else {
    // TODO: should this be an error?
    return Ok(None);
  };

  Ok(Some(Self {
    identifier,
    ty,
    span: aster.finish_span(start),
  }))
});

impl_ast!(FunctionArguments: (compiler, aster, start) => {
  let mut arguments = vec![];

  aster.reader.push_mark();

  loop {
    // Read argument, if there is one to read
    let Some(argument) = aster.make(compiler)? else {
      // Otherwise, return our arguments
      aster.reader.pop_mark();
      break;
    };

    aster.reader.drop_mark();
    arguments.push(argument);

    // Push a mark in case there is no comma following the argument
    aster.reader.push_mark();
    aster.reader.seek_whitespace_and_comments();

    let Some(TokenKind::Punctuation(Punctuation::Comma)) = aster.reader.peek_kind() else {
      // If there is none, pop the mark and return our arguments
      aster.reader.pop_mark();
      break;
    };

    // Otherwise, drop the mark and continue onto the next argument
    aster.reader.drop_mark();
    aster.reader.push_mark();
    aster.reader.seek_whitespace_and_comments();
  };

  Ok(Some(Self {
    arguments,
    span: aster.finish_span(start),
  }))
});

impl_ast!(Function: (compiler, aster, start) => {
  // Parse function name
  let Some(identifier) = aster.make(compiler)? else {
    return Ok(None);
  };

  // Push a mark in case this function has an implicit return type
  aster.reader.seek_whitespace_and_comments();
  aster.reader.push_mark();

  let return_ty = {
    if let Some(TokenKind::Punctuation(Punctuation::RightArrow)) = aster.reader.next_kind() {
      // A return type is indicated here with a right arrow before the type.
      // Drop the mark and parse the type
      aster.reader.drop_mark();
      aster.reader.seek_whitespace_and_comments();

      let Some(ty) = aster.make(compiler)? else {
        return ExpectedSnafu { what: What::Type }.fail()?;
      };

      Some(ty)
    } else {
      // Otherwise, pop the mark and go on to parse the arguments and body
      aster.reader.pop_mark();

      None
    }
  };

  // Push a mark in case this function has no arguments
  aster.reader.seek_whitespace_and_comments();
  aster.reader.push_mark();

  let arguments = {
    if let Some(TokenKind::Punctuation(Punctuation::Colon)) = aster.reader.next_kind() {
      // Arguments are indicated here with a colon preceeding them.  Drop the
      // mark and parse the arguments
      aster.reader.drop_mark();
      aster.reader.seek_whitespace_and_comments();

      let Some(arguments) = aster.make(compiler)? else {
        return ExpectedSnafu { what: What::FunctionArguments }.fail()?;
      };

      Some(arguments)
    } else {
      // Otherwise, pop the mark and go on to parse the body

      None
    }
  };

  aster.reader.seek_whitespace_and_comments();

  // Finally parse body
  let Some(body) = aster.make(compiler)? else {
    return ExpectedSnafu { what: What::FunctionBody }.fail()?;
  };

  Ok(Some(Self {
    identifier,
    return_ty,
    arguments,
    body,
    span: aster.finish_span(start),
  }))
});
