mod make;
pub mod bufreader;
pub mod rereader;
pub mod pprint;

use std::fs::File;

use bufreader::BufferedUtf8MetadataReader;
use crate::aster::rereader::Rereader;
use crate::string_pool::StringPool;

use crate::tokenize::{self, Tokenizer};
use crate::tokenize::token::Span;
use crate::lang::Lazy;
use crate::lang::module::ModuleId;

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

pub fn asterize<'pool>(lazy: &mut Lazy<'pool>, pool: &'pool StringPool, id: ModuleId) -> Result<(), Error> {
  let path = lazy.get_path(id).to_owned();
  let file = File::open(&path).expect("failed to open path");
  let meta_reader = BufferedUtf8MetadataReader::<64, _>::new(file);
  let tokenizer = Tokenizer::new(pool, id, meta_reader);
  let mut rereader = Rereader::new(tokenizer, id);

  make::make(lazy, &mut rereader)
}
