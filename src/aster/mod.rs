mod make;
pub mod bufreader;
pub mod rereader;
pub mod pprint;

use std::fs::File;

use bufreader::BufferedUtf8MetadataReader;
use crate::aster::rereader::Rereader;
use crate::lang::reference::{ModuleReference, Store};
use crate::string_pool::StringPool;

use crate::tokenize::Tokenizer;
use crate::tokenize::{self, token::Span};
use crate::lang::Lazy;

#[derive(Debug)]
pub enum Error {
  Token(tokenize::Error),
  Expected {
    what: &'static str,
    at: Span,
  },
  Invalid {
    what: &'static str,
    at: Span,
  },
}

pub fn asterize<'pool>(lazy: &mut Lazy<'pool>, pool: &'pool StringPool, module: ModuleReference) -> Result<(), Error> {
  let path = lazy.get_path(module).path.as_path();
  let file = File::open(path).expect("failed to open path");
  let meta_reader = BufferedUtf8MetadataReader::<64, _>::new(file);
  let tokenizer = Tokenizer::new(pool, module, meta_reader);
  let mut rereader = Rereader::new(tokenizer, module);

  let result = make::make(lazy, &mut rereader);

  let tokens_id = lazy.get_path(module).tokens;
  for token_span in rereader.examine_tokens() {
    lazy.rget_mut(tokens_id).push(token_span);
  };

  result
}
