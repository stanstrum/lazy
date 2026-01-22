mod string_pool;
mod lang;
mod tokenize;
mod aster;
mod resolve;
mod error;

use std::process::ExitCode;

use lang::Lazy;

use crate::lang::module::ModuleId;
use crate::string_pool::StringPool;

use crate::aster::pprint::Pretty;

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

  let error_handler: Result<(), error::PrintableMesage> = 'error: {
    if let Err(err) = aster::asterize(&mut lazy, &pool, global) {
      break 'error Err(err.into());
    };

    if let Err(err) = resolve::resolve(&mut lazy, global) {
      break 'error Err(err.into());
    };

    Ok(())
  };

  // dbg!(&lazy);

  let source = lazy[global].print(&lazy)
    .map(|s| format!(line_dbg!("{}"), s))
    .collect::<Vec<_>>()
    .join("\n");
  println!("{source}");

  match error_handler {
    Ok(()) => ExitCode::SUCCESS,
    Err(message) => {
      error::print_message(&lazy, message);
      ExitCode::FAILURE
    },
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn debug_tokens() {
    let pool = StringPool::new();
    let (lazy, global) = setup(&pool);
    let path = lazy.get_path(global).path.as_path();

    let file = std::fs::File::open(path).unwrap();
    let meta_reader= aster::bufreader::BufferedUtf8MetadataReader::<64, _>::new(file);
    let tokens = tokenize::Tokenizer::<'_, 64, _>::new(&pool, global, meta_reader);
    let rereader = crate::aster::rereader::Rereader::new(tokens, global);

    let mut indentation = 0isize;
    for (i, token) in rereader.enumerate() {
      let token = match token {
        Ok((token, _)) => token,
        Err(err) => panic!("err: {err:?}"),
      };

      let padding = " ".repeat(indentation as _);
      print!("{i:<2}: [{indentation:>+3}] {padding}");

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
