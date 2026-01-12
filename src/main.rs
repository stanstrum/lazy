mod string_pool;
mod lang;
mod tokenize;
mod aster;

use std::process::ExitCode;

use lang::Lazy;

use crate::aster::asterize;
use crate::string_pool::StringPool;

fn main() -> ExitCode {
  let pool = StringPool::new();
  let mut lazy = Lazy::new(&pool);

  let cwd = std::env::current_dir().expect("cwd failed");
  let input_path = cwd.join("snippets/00_base_main.zy");
  let global = lazy.add_file("global", input_path);

  if asterize(&mut lazy, &pool, global).is_err() {
    return ExitCode::FAILURE;
  };

  dbg!(lazy);

  ExitCode::SUCCESS
}
