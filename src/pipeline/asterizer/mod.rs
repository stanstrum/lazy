pub mod ast;
pub(crate) mod error;
mod reader;

use std::fmt::Debug;
use std::marker::PhantomData;

use ast::TopLevelNamespace;
use reader::TokenReader;
use typename::TypeName;

use crate::compiler::error::ReadSpan;
use crate::compiler::{Asterize, Compiler, CompilerStoreHandle, CompilerWorkflow};
use crate::tokenizer::{Span, SpanStart, Token};
use crate::Result;

#[macro_export]
macro_rules! impl_ast {
  ($what:ident: @todo) => {
    impl_ast!($what: (_, _, _) => todo!());
  };

  ($what:ident: @stub) => {
    impl_ast!($what: (_, _, _) => {
      warn!("{}: Ast::make stubbed for {}", $crate::enchant!("stub"), Self::better_type_name());

      Ok(None)
    });
  };

  ($what:ident: ($compiler:tt, $aster:tt, $start:tt) => $expr:expr) => {
    impl<W: CompilerWorkflow> Ast<W> for $what<W> {
      fn make($compiler: &mut Compiler<W>, $aster: &mut Asterizer<W>, $start: SpanStart<W>) -> Result<Option<Self>> {
        $expr
      }
    }
  };
}

#[derive(Debug)]
pub(crate) struct Asterizer<W: CompilerWorkflow> {
  /// The reader through which Tokens can be read programmatically
  reader: TokenReader<W>,
  marker: PhantomData<W>,
}

/// The interface through which AST objects are created and identified by Span
trait Ast<W: CompilerWorkflow>: TypeName + Debug + Sized {
  /// Attempts to parse tokens from TokenReader into Self if possible.  Errors
  /// are only emitted if input is absolutely unparseable, otherwise Ok(None) is
  /// returned.
  fn make(
    compiler: &mut Compiler<W>,
    aster: &mut Asterizer<W>,
    start: SpanStart<W>,
  ) -> Result<Option<Self>>;
  // /// Returns the Span pertaining to Self, for error message purposes
  // fn get_span(&self) -> Span;

  fn better_type_name() -> String {
    Self::type_name()
      .strip_prefix("lazy::pipeline::")
      .unwrap()
      .strip_suffix("<lazy::compiler::workflow::DefaultWorkflow>")
      .unwrap()
      .into()
  }
}

impl<W: CompilerWorkflow> Asterizer<W> {
  /// Main interface through which AST object are parsed.  This method provides
  /// additional information to the associated Ast::make methods so debug
  /// information may be preserved.
  fn make<T: Ast<W>>(&mut self, compiler: &mut Compiler<W>) -> Result<Option<T>> {
    // let type_name = T::better_type_name();
    // trace!("{}: Ast::make: {:#?}", type_name, self.reader.peek());

    let marks_len_before = self.reader.marks_len();
    let start = self.reader.get_start();

    // Push a mark in case this operation fails
    self.reader.push_mark();

    let result = T::make(compiler, self, start)?;

    // Deal with the mark according to the status of the result
    if result.is_some() {
      // The object was parsed successfully, so drop the mark
      self.reader.drop_mark();
    } else {
      // The object failed to parse, so pop the mark
      self.reader.pop_mark();
    };

    // trace!("{}: Ast::make: {result:#?}", type_name);

    let marks_len_after = self.reader.marks_len();
    assert!(marks_len_before == marks_len_after, "mark length mismatch!");

    Ok(result)
  }

  /// Creates a Span using the SpanStart provided as a parameter of Ast::make
  fn finish_span(&self, start: SpanStart<W>) -> Span<W> {
    start.into_span(self.reader.get_position())
  }

  /// Creates a ReadSpan using the next Token in the stream
  fn next_read_span(&self, compiler: &mut Compiler<W>) -> Result<ReadSpan> {
    compiler.span_to_read_span(self.reader.next_span())
  }
}

impl<W: CompilerWorkflow> Asterize<W> for Asterizer<W> {
  type In = Vec<Token<W>>;
  type Out = ast::TopLevelNamespace<W>;

  fn new(tokens: Self::In, handle: CompilerStoreHandle<W>) -> Self {
    Self {
      reader: TokenReader::new(tokens, handle),
      marker: Default::default(),
    }
  }

  fn asterize(mut self, compiler: &mut Compiler<W>) -> Result<Self::Out> {
    let top_level = {
      if let Some(top_level) = self.make(compiler)? {
        top_level
      } else {
        TopLevelNamespace::new_empty(self.reader.get_start())
      }
    };

    Ok(top_level)
  }
}
