mod make;
pub mod rereader;
pub mod pprint;

use std::fs::File;
use std::path::PathBuf;

use rereader::Rereader;
use ::tokenize::bufreader::BufferedUtf8MetadataReader;
use tokenize::{Tokenizer, self};
use ::lang::{Compiler, CompilerPoolStore};
use ::lang::reference::Store;
use ::lang::span::Span;

pub type Error<C> = ::lang::error::AsterError<C>;

pub fn asterize<'lazy, 'pool, C: Compiler>(store: &'lazy mut C::Store<'pool>, module: C::ModuleReference) -> Result<(), Error<C>>
  where 'lazy: 'pool
{
  let path = store.get_path(module).path.as_path();
  let file = File::open(path).expect("failed to open path");
  let meta_reader = BufferedUtf8MetadataReader::<64, _>::new(file);
  let name = store.describe_module(module);
  let tokenizer = Tokenizer::new(store.pool(), module, name, meta_reader);
  let mut rereader = Rereader::new(tokenizer, module);

  let result = make::make(store, &mut rereader);

  let tokens_id = store.get_path(module).tokens;
  let tokens_borrow = store.rget_mut(tokens_id);

  for token_span in rereader.examine_tokens() {
    tokens_borrow.push(token_span);
  };

  result
}

