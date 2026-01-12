mod bufreader;
mod string_pool;
mod lang;
mod tokenize;
mod aster;

use lang::Lazy;

use crate::aster::asterize;
use crate::string_pool::StringPool;

fn main() {
  let pool = StringPool::new();
  let mut lazy = Lazy::new(&pool);

  let cwd = std::env::current_dir().expect("cwd failed");
  let input_path = cwd.join("snippets/00_base_main.zy");
  let global = lazy.add_file("global", input_path);

  if let Err(err) = asterize(&mut lazy, &pool, global) {
    dbg!(err);
  };

  dbg!(lazy);
}
