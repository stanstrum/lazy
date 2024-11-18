use std::path::PathBuf;
use std::process::{Command, Stdio};
use tempfile::NamedTempFile;

use crate::compiler::{Compiler, workflow::DefaultWorkflow};
use crate::arg_parser;
use crate::enchant;

macro_rules! compile_test {
  ($ident:ident) => {
    compile_test!($ident = stringify!($ident));
  };

  ($ident:ident = $test_name:expr) => {
    #[test]
    fn $ident() {
      logger_init();

      let input_path = PathBuf::from(
        concat!(env!("CARGO_MANIFEST_DIR"),
        "/snippets/",
        $test_name,
        ".zy",
      ));

      let input_path = input_path.to_string_lossy();

      let output_file = NamedTempFile::new().expect("failed to make temp file")
        .into_temp_path();
      let output_path = output_file.to_string_lossy().into_owned();

      let args = [
        "--input", &input_path,
        "--output", &output_path,
        "--print-llvm",
      ];

      let error_harness = || {
        let options = arg_parser::parse(args)?;
        let mut compiler = Compiler::<DefaultWorkflow>::new(options)?;
        compiler.compile()
      };

      if let Err(err) = error_harness() {
        let message = err.to_string();

        err.output_to_logger();
        panic!("compilation failed: {message}");
      };

      let executable = output_file.keep()
        .expect("failed to keep temp file");

      let mut command = Command::new(executable);
      command.stderr(Stdio::piped());
      command.stdout(std::io::stdout());

      let mut child = command.spawn()
        .expect("failed to spawn child process");
      let id = child.id();

      let exit_code = child.wait()
        .expect("failed to wait on child process");

      debug!("{}: {output_path}: process {id} returned {exit_code}", enchant!("test"));
      assert!(exit_code.success(), "{output_path}: process {id} returned {exit_code}");
    }
  };
}

fn logger_init() {
  static mut LOGGER_IS_INITIALZIED: bool = false;
  eprintln!();

  unsafe {
    if LOGGER_IS_INITIALZIED {
      return;
    };

    crate::logger::init();
    LOGGER_IS_INITIALZIED = true;
  };
}

compile_test!(base_main = "00_base_main");
compile_test!(assn = "01_assn");
compile_test!(hello_world = "02_hello_world");
compile_test!(trait_imp = "03_trait_imp");
compile_test!(extended_operators = "04_extended_operators");
compile_test!(counter_ns = "05_counter_ns");
compile_test!(type_alias = "06_type_alias");
compile_test!(struct_stuff = "07_struct_stuff");
compile_test!(codegen = "08_codegen");
compile_test!(r#extern = "09_extern");
compile_test!(read_source = "10_read_source");
compile_test!(import_std = "11_import_std");
compile_test!(structs = "12_structs");
compile_test!(struct_generic = "13_struct_generic");
compile_test!(slice = "14_slice");
compile_test!(namespace = "15_namespace");
compile_test!(namespaces = "16_namespaces");
compile_test!(string_and_char_escapes = "17_string_and_char_escapes");
compile_test!(control_flow = "18_control_flow");
compile_test!(class_methods = "19_class_methods");
compile_test!(hang = "20_hang");
compile_test!(bare_bones);
compile_test!(counter);
compile_test!(r#if = "if");
compile_test!(message);
