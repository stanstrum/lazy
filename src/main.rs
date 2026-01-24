mod string_pool;

mod lang;
mod tokenize;
mod aster;
mod resolve;

mod error;
mod settings;

use std::process::ExitCode;

use lang::Lazy;

use crate::string_pool::StringPool;

use crate::aster::pprint::Pretty;

fn main() -> ExitCode {
  let args = std::env::args();
  let (settings, verb) = match settings::parse_and_display(args) {
    Ok(settings) => settings,
    Err(exit_code) => return exit_code,
  };

  let pool = StringPool::new();
  let mut lazy = Lazy::new(&pool, settings);

  let path = lazy.settings.input_path.to_owned();
  let global = lazy.add_file("global", path);

  let error_handler: Result<(), error::PrintableMesage> = 'error: {
    if let Err(err) = aster::asterize(&mut lazy, &pool, global) {
      break 'error Err(err.into());
    };

    if let Err(err) = resolve::resolve(&mut lazy, global) {
      break 'error Err(err.into());
    };

    println!("todo: typeck");
    println!("todo: generate");

    match verb {
      settings::Verb::Check => {
        let source = lazy[global].print(&lazy)
          .map(|s| format!(line_dbg!("{}"), s))
          .collect::<Vec<_>>()
          .join("\n");
        println!("{source}");
      },
      settings::Verb::Build => todo!("build"),
      settings::Verb::Run => todo!("run"),
    };

    Ok(())
  };

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
  use std::path::PathBuf;
  use crate::settings::Settings;
  use crate::error::Level;

  use super::*;

  #[test]
  fn debug_tokens() {
    let settings = Settings {
      executable: "lazy:test".into(),
      input_path: PathBuf::from("snippets/00_base_main.zy"),
      output_path: "a.out".into(),
      log_level: Level::Debug,
    };

    let pool = StringPool::new();
    let mut lazy = Lazy::new(&pool, settings);

    let global = lazy.add_file("global", lazy.settings.input_path.to_owned());
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
