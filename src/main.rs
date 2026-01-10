mod bufreader;
mod string_pool;
mod lang;

use std::fs::File;

use lang::Lazy;
use bufreader::BufferedUtf8MetadataReader;

fn main() {
  let lazy = Lazy::new();

  let cwd = std::env::current_dir().expect("cwd failed");
  let input_path = cwd.join("snippets/00_base_main.zy");

  let file = File::open(input_path).unwrap();
  let mut reader = BufferedUtf8MetadataReader::<4>::new(file);

  while let Some(Ok(ch)) = reader.next() {
    print!("{ch}");
  };

  dbg!(lazy);
}
