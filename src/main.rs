mod bufreader;
mod string_pool;
mod lang;
mod tokenize;

use lang::Lazy;

use crate::bufreader::BufferedUtf8MetadataReader;
use crate::tokenize::{Token, Tokenizer};
use crate::string_pool::StringPool;

fn main() {
  let pool = StringPool::new();
  let mut lazy = Lazy::new(&pool);

  let cwd = std::env::current_dir().expect("cwd failed");
  let input_path = cwd.join("snippets/00_base_main.zy");
  let global = lazy.add_file("global", input_path);
  let file = lazy.open_file(global);
  let meta_reader = BufferedUtf8MetadataReader::<64, _>::new(file);
  let mut tokenizer = Tokenizer::new(&pool, global, meta_reader);
  let mut indentation = 0;
  while let Some(result) = tokenizer.next() {
    match result {
      Ok(tok) => {
        print!(
          "({indentation}) {}:{} - {}:{} = ",
          tok.start.line,
          tok.start.column,
          tok.end.line,
          tok.end.column,
        );
        match &tok.tok {
          Token::Identifier(id) => println!("Identifier({id:?}: {:?})", pool.get(*id).collect::<String>()),
          other => println!("{other:?}"),
        };

        if let Token::Indent(difference) = &tok.tok {
          indentation += *difference;
        };
      },
      Err(err) => panic!("error: {err:?}"),
    };
  };

  dbg!(lazy);
}
