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

compile_test!("00_base_main": base_main);
compile_test!("00_basic": basic);
compile_test!("01_assn": assn);
compile_test!("02_hello_world": hello_world);
compile_test!("03_trait_imp": trait_imp);
compile_test!("04_extended_operators": extended_operators);
compile_test!("05_counter_ns": counter_ns);
compile_test!("06_type_alias": type_alias);
compile_test!("07_struct_stuff": struct_stuff);
compile_test!("08_codegen": codegen);
compile_test!("09_extern": r#extern);
compile_test!("10_read_source": read_source);
compile_test!("11_import_std": import_std);
compile_test!("12_structs": structs);
compile_test!("13_struct_generic": struct_generic);
compile_test!("14_slice": slice);
compile_test!("15_namespace": namespace);
compile_test!("16_namespaces": namespaces);
compile_test!("17_string_and_char_escapes": string_and_char_escapes);
compile_test!("18_control_flow": control_flow);
compile_test!("19_class_methods": class_methods);
compile_test!("20_hang": hang);
compile_test!("21_operator_coverage": operator_coverage);
compile_test!("22_import_26": import_26);
compile_test!("23_struct_mod": struct_mod);
compile_test!("bare_bones": bare_bones);
compile_test!("counter": counter);
compile_test!("if": r#if);
compile_test!("message": message);
compile_test!("std_future": std);

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
  let mut lazy = lazy::Lazy::new(&pool, settings);

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
