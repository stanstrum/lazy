mod string_pool;
mod lang;
mod tokenize;
mod aster;

use std::process::ExitCode;

use lang::Lazy;

use crate::aster::asterize;
use crate::lang::module::ModuleId;
use crate::string_pool::StringPool;

fn setup(pool: &StringPool) -> (Lazy, ModuleId) {
  let mut lazy = Lazy::new(pool);

  let cwd = std::env::current_dir().expect("cwd failed");
  let input_path = cwd.join("snippets/00_base_main.zy");
  let global = lazy.add_file("global", input_path);

  (lazy, global)
}

fn main() -> ExitCode {
  let pool = StringPool::new();
  let (mut lazy, global) = setup(&pool);

  if asterize(&mut lazy, &pool, global).is_err() {
    return ExitCode::FAILURE;
  };

  dbg!(lazy);

  ExitCode::SUCCESS
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn debug_tokens() {
    let pool = StringPool::new();
    let (mut lazy, global) = setup(&pool);
    let path = lazy.get_path(global);

    let file = std::fs::File::open(path).unwrap();
    let meta_reader= aster::bufreader::BufferedUtf8MetadataReader::<64, _>::new(file);
    let tokens = tokenize::Tokenizer::<'_, 64, _>::new(&pool, global, meta_reader);

    let mut indentation = 0isize;
    for (i, token) in tokens.enumerate() {
      let token = match token {
        Ok((token, _)) => token,
        Err(err) => panic!("err: {err:?}"),
      };

      print!("{i:<2}: [{indentation:>+3}] ");

      match &token {
        tokenize::token::Token::Identifier(id) => println!("Identifier({:?})", pool.get(*id).collect::<String>()),
        other => println!("{other:?}"),
      };

      if let tokenize::token::Token::Indent(difference) = token {
        indentation += difference;
      };
    };
  }
}
