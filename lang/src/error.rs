use std::path::PathBuf;

use crate::span::Span;
use crate::Compiler;

#[derive(Debug)]
pub enum LazyError<C: Compiler> {
  NotExist(PathBuf),
  Aster(AsterError<C>),
}

#[derive(Debug)]
pub enum TokenError<C: Compiler> {
  IO {
    module: C::ModuleReference,
    name: String,
  },
  InvalidNumeric {
    span: Span<C>,
  },
}

#[derive(Debug)]
pub enum AsterError<C: Compiler> {
  Token(TokenError<C>),
  Lazy(Box<LazyError<C>>),
  Expected {
    what: &'static str,
    at: Span<C>,
  },
  Invalid {
    what: &'static str,
    at: Span<C>,
  },
}

#[derive(Debug)]
pub struct ResolveError<C: Compiler> {
  pub base: ResolveErrorBase<C>,
  pub call_stack: String,
}

#[derive(Debug)]
pub enum ResolveErrorBase<C: Compiler> {
  MissingEntryPoint {
    module_name: String,
    file: C::ModuleReference,
  },
  UnknownTypeName {
    module_name: String,
    span: Span<C>,
  },
  TypeMismatch {
    whence: &'static str,
    a_print: String,
    a_span: Span<C>,
    b_print: String,
    b_span: Span<C>,
  },
  UnresolvedInVerify {
    what: String,
    span: Span<C>,
  },
  BadQualify {
    span: Span<C>,
  },
  NotImplemented {
    what: &'static str,
    span: Span<C>,
  },
  Lazy(Box<LazyError<C>>),
}

// SPONGE: move this to gluezy
impl<C: Compiler> From<LazyError<C>> for AsterError<C> {
  fn from(value: LazyError<C>) -> Self {
    match value {
      value @ LazyError::NotExist(_) => Self::Lazy(Box::new(value)),
      LazyError::Aster(error) => error,
    }
  }
}

impl<C: Compiler> From<AsterError<C>> for LazyError<C> {
  fn from(value: AsterError<C>) -> Self {
    Self::Aster(value)
  }
}
