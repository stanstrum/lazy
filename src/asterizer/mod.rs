pub mod ast;
mod reader;
pub(crate) mod errors;

use crate::Result;
use crate::compiler::{
  CompilerWorkflow,
  Asterize,
  Compiler,
};
use crate::tokenizer::{
  Span,
  SpanStart,
  Token,
};

use ast::TopLevelNamespace;
pub(self) use reader::TokenReader;
use errors::*;

pub(super) struct Asterizer;

/// The interface through which AST objects are created and identified by Span
trait Ast<W: CompilerWorkflow> where Self: Sized {
  /// Attempts to parse tokens from TokenReader into Self if possible.  Errors
  /// are only emitted if input is absolutely unparseable, otherwise Ok(None) is
  /// returned.
  fn make(compiler: &mut Compiler<W>, reader: &mut TokenReader, start: SpanStart) -> Result<Option<Self>>;
  /// Returns the Span pertaining to Self, for error message purposes
  fn get_span(&self) -> Span;
}

impl Asterizer {
  /// Main interface through which AST object are parsed.  This method provides
  /// additional information to the associated Ast::make methods so debug
  /// information may be preserved.
  fn make<
    W: CompilerWorkflow,
    T: Ast<W>,
  >(&self, compiler: &mut Compiler<W>, reader: &mut TokenReader) -> Result<Option<T>> {
    let start = reader.get_start();

    // Push a mark in case this operation fails
    reader.push_mark();

    let result = T::make(compiler, reader, start)?;

    // Deal with the mark according to the status of the result
    if result.is_some() {
      // The object was parsed successfully, so drop the mark
      reader.drop_mark();
    } else {
      // The object failed to parse, so pop the mark
      reader.pop_mark();
    };

    Ok(result)
  }
}

impl<W: CompilerWorkflow> Asterize<W> for Asterizer {
  type In = Vec<Token>;
  type Out = ast::TopLevelNamespace;

  fn new() -> Self {
    Self
  }

  fn asterize(self, compiler: &mut Compiler<W>, tokens: Self::In) -> Result<Self::Out> {
    let mut reader = TokenReader::new(tokens);

    let top_level = {
      if let Some(top_level) = self.make(compiler, &mut reader)? {
        top_level
      } else {
        TopLevelNamespace::new_empty(reader.get_start())
      }
    };

    Ok(top_level)
  }
}
