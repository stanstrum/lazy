mod make;
pub mod rereader;
pub mod pprint;

use std::fs::File;

use rereader::Rereader;
use ::tokenize::bufreader::BufferedUtf8MetadataReader;
use tokenize::{Tokenizer, self};
use ::lang::Compiler;
use ::lang::reference::Store;
use ::lang::span::Span;

#[derive(Debug)]
pub enum Error<C: Compiler> {
  Token(tokenize::Error<C>),
  // Lazy(Box<crate::lang::LazyError>),
  Expected {
    what: &'static str,
    at: Span<C>,
  },
  Invalid {
    what: &'static str,
    at: Span<C>,
  },
}

pub fn asterize<C: Compiler>(store: &mut C::Store<'_>, module: C::ModuleReference) -> Result<(), Error<C>> {
  let path = store.get_path(module).path.as_path();
  let file = File::open(path).expect("failed to open path");
  let meta_reader = BufferedUtf8MetadataReader::<64, _>::new(file);
  let name = store.describe_module(module);
  let tokenizer = Tokenizer::new(store.pool, module, name, meta_reader);
  let mut rereader = Rereader::new(tokenizer, module);

  let result = make::make(store, &mut rereader);

  let tokens_id = store.get_path(module).tokens;
  let tokens_borrow = store.rget_mut(tokens_id);

  for token_span in rereader.examine_tokens() {
    tokens_borrow.push(token_span);
  };

  result
}

// SPONGE: move this to gluezy
impl From<crate::lang::LazyError> for Error {
  fn from(value: crate::lang::LazyError) -> Self {
    match value {
      value @ crate::lang::LazyError::NotExist(_) => Self::Lazy(Box::new(value)),
      crate::lang::LazyError::Aster(error) => error,
    }
  }
}
