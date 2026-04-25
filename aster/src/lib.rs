mod make;
pub mod rereader;
pub use ::pprint::*;

use std::fs::File;

use rereader::Rereader;
use ::tokenize::bufreader::BufferedUtf8MetadataReader;
use tokenize::{Tokenizer, self};
use ::lang::{Compiler, CompilerPoolStore};
use ::lang::reference::Store;

pub type Error<C> = ::lang::error::AsterError<C>;

pub fn asterize<'pool, C: Compiler>(store: &mut C::Store<'pool>, module: C::ModuleReference) -> Result<(), Error<C>> {
  let path = store.get_path(module).path.as_path();
  let file = File::open(path).expect("failed to open path");
  let meta_reader = BufferedUtf8MetadataReader::<64, _>::new(file);
  let name = store.describe_module(module);
  let tokenizer = Tokenizer::<'_, C, 64, _>::new(store.pool(), module, name, meta_reader);
  let mut rereader = Rereader::<'_, C, 64, _>::new(tokenizer, module);

  let result = make::make(store, &mut rereader);

  let tokens_id = store.get_path(module).tokens;
  let tokens_borrow = store.rget_mut(tokens_id);

  for token_span in rereader.examine_tokens() {
    tokens_borrow.push(token_span);
  };

  result
}

