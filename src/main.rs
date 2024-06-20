mod compiler;
mod tokenizer;
mod asterizer;
mod error;

mod debug;
mod colors;

use error::*;

fn compile(args: Vec<String>) -> Result<(), CompilationError> {
  let Some(input_file_path) = args.get(1) else {
    return ArgumentSnafu {
      message: "No input file provided."
    }.fail();
  };

  let mut compiler = compiler::Compiler::new(input_file_path.into());

  compiler.compile()
}

fn main() {
  let args: Vec<String> = std::env::args().collect();

  if let Err(error) = compile(args) {
    match &error {
      CompilationError::Argument { .. } => {
        eprintln!("Argument error: {error}");
      },
      CompilationError::InputFile { .. } => {
        eprintln!("Input file error: {error}");
      },
      _ => {}
    };

    std::process::exit(1);
  };
}

#[cfg(test)]
mod tests {
  use super::*;

  macro_rules! compile_snippet {
    ($test_name:ident, $source:literal) => {
      #[test]
      fn $test_name() {
        let args = vec![
          "test_suite",
          concat!(
            "snippets/",
            $source,
            ".zy"
          )
        ];

        let args = args
          .into_iter()
          .map(String::from)
          .collect();

        if let Err(err) = compile(args) {
          panic!("compilation failed: {err}");
        };
      }
    };
  }

  compile_snippet!(base_main, "00_base_main");
  compile_snippet!(assn, "01_assn");
  compile_snippet!(hello_world, "02_hello_world");
  compile_snippet!(trait_imp, "03_trait_imp");
  compile_snippet!(extended_operators, "04_extended_operators");
  compile_snippet!(counter_ns, "05_counter_ns");
  compile_snippet!(type_alias, "06_type_alias");
  compile_snippet!(struct_stuff, "07_struct_stuff");
  compile_snippet!(codegen, "08_codegen");
  compile_snippet!(r#extern, "09_extern");
  compile_snippet!(read_source, "10_read_source");
  compile_snippet!(import_std, "11_import_std");
  compile_snippet!(structs, "12_structs");
  compile_snippet!(struct_generic, "13_struct_generic");
  compile_snippet!(slice, "14_slice");
  compile_snippet!(namespaces, "15_namespace");
  compile_snippet!(string_escapes, "17_string_and_char_escapes");
  compile_snippet!(control_flow, "18_control_flow");
  compile_snippet!(bare_bones, "bare_bones");
  compile_snippet!(counter, "counter");
  compile_snippet!(r#if, "if");
  compile_snippet!(message, "message");
  compile_snippet!(namespace_module, "16_namespaces");
  // compile_snippet!(std, "std");
  compile_snippet!(class_methods, "19_class_methods");
}
