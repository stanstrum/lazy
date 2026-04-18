use std::path::PathBuf;
use crate::settings::Settings;
use crate::error::Level;

use super::*;

macro_rules! compile_test {
  ($file:literal: $test:ident) => {
    #[test]
    fn $test() {
      let out_handle = tempfile::Builder::new()
        .tempfile()
        .expect("to create tempfile")
        .into_temp_path();

      let out_path = out_handle.to_string_lossy();

      let exit = run_with(
        [
          "lazy:test",
          "--log-level=debug",
          &format!("--output-file={out_path}"),
          "run",
          concat!(env!("CARGO_MANIFEST_DIR"), "/snippets/", $file)
        ]
          .map(String::from)
          .into_iter()
      );

      out_handle.close().expect("to close tempfile");

      assert!(exit == ExitCode::SUCCESS);
    }
  };
}

compile_test!("00_base_main.zy": base_main);
compile_test!("00_basic.zy": basic);
compile_test!("01_assn.zy": assn);
compile_test!("02_hello_world.zy": hello_world);
compile_test!("03_trait_imp.zy": trait_imp);
compile_test!("04_extended_operators.zy": extended_operators);
compile_test!("05_counter_ns.zy": counter_ns);
compile_test!("06_type_alias.zy": type_alias);
compile_test!("07_struct_stuff.zy": struct_stuff);
compile_test!("08_codegen.zy": codegen);
compile_test!("09_extern.zy": r#extern);
compile_test!("10_read_source.zy": read_source);
compile_test!("11_import_std.zy": import_std);
compile_test!("12_structs.zy": structs);
compile_test!("13_struct_generic.zy": struct_generic);
compile_test!("14_slice.zy": slice);
compile_test!("15_namespace.zy": namespace);
compile_test!("16_namespaces.zy": namespaces);
compile_test!("17_string_and_char_escapes.zy": string_and_char_escapes);
compile_test!("18_control_flow.zy": control_flow);
compile_test!("19_class_methods.zy": class_methods);
compile_test!("20_hang.zy": hang);
compile_test!("21_operator_coverage.zy": operator_coverage);
compile_test!("bare_bones.zy": bare_bones);
compile_test!("counter.zy": counter);
compile_test!("if.zy": r#if);
compile_test!("message.zy": message);
compile_test!("std.zy": std);

#[test]
fn debug_tokens() {
  let settings = Settings {
    executable: "lazy:test".into(),
    input_path: PathBuf::from("snippets/00_base_main.zy"),
    output_path: "a.out".into(),
    log_level: Level::Debug,
    argv: vec!["lazy:test", "ck:test", "snippets/00_base_main.zy"].into_iter().map(String::from).collect()
  };

  let pool = StringPool::new();
  let mut lazy = Lazy::new(&pool, settings);

  let global = lazy.add_file("global", lazy.settings.input_path.to_owned(), None)
    .expect("to add global module");
  let path = lazy.get_path(global).path.as_path();

  let file = std::fs::File::open(path).unwrap();
  let meta_reader= aster::bufreader::BufferedUtf8MetadataReader::<64, _>::new(file);
  let name = lazy.describe_module(global);
  let tokens = tokenize::Tokenizer::<'_, 64, _>::new(&pool, global, name, meta_reader);
  let rereader = crate::aster::rereader::Rereader::new(tokens, global);

  let mut indentation = 0isize;
  for (i, token) in rereader.enumerate() {
    let (token, span) = match token {
      Ok(token) => token,
      Err(err) => panic!("err: {err:?}"),
    };

    let padding = " ".repeat(indentation as _);
    print!("{i:<2}: [{indentation:>+3}] {padding}");

    match &token {
      tokenize::token::Token::Identifier(id) => print!("Identifier({:?})", pool.get(*id)),
      other => print!("{other:?}"),
    };

    if let tokenize::token::Token::Indent(difference) = token {
      print!(" (indent: {})", span.start.indentation);
      indentation += difference;
    };

    println!();
  };
}
