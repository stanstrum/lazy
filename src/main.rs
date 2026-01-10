mod bufreader;
mod string_pool;
mod lang;

use lang::Lazy;
use bufreader::BufferedUtf8MetadataReader;

fn main() {
  let mut lazy = Lazy::new();

  let cwd = std::env::current_dir().expect("cwd failed");
  let input_path = cwd.join("snippets/00_base_main.zy");
  let global = lazy.add_file("global", input_path);

  let file = lazy.open_file(global);
  let mut reader = BufferedUtf8MetadataReader::<64, _>::new(file);

  while let Some(Ok(ch)) = reader.next() {
    print!("{ch}");
  };

  dbg!(lazy);
}
