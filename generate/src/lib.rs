mod context;
pub mod args;
mod compile;

mod types;

use std::collections::HashMap;
use std::path::Path;

use gluezy::LazyStructures;
pub use inkwell::targets::FileType;
use lang::Compiler;
use lazy_macros::{line_dbg, print_message};
use lang::span::GetSpan;
use log::{Level, MessageContents, MessageSection, WithinSource};
use lang::reference::{Reference, Store, TypeReference};
use lang::span::Span;
use lang::ty::{Type, TypeOf, TypeValue};

use {args::*, context::*};

#[derive(Debug)]
struct ResolvedType<C: Compiler> {
  pub reference: TypeReference<C>,
  pub ty: TypeValue<C>,
}

fn expect_resolved_type<C: Compiler>(store: &C::Store<'_>, t: &(impl TypeOf<C> + GetSpan<C>)) -> Result<C, ResolvedType<C>> {
  let Some(ty) = t.type_of(store) else {
    let span = t.get_span(store);

    return Err(Error::StillUnresolved {
      what: line_dbg!("type").into(),
      note: "here".into(),
      span,
    });
  };

  todo!()
}

#[derive(Debug)]
pub enum Error<C: Compiler> {
  LLVMError(String),
  StillUnresolved {
    what: String,
    note: String,
    span: Span<C>,
  },
}

type Result<C, T = ()> = std::result::Result<T, Error<C>>;

impl<C: Compiler> From<inkwell::support::LLVMString> for Error<C> {
  fn from(value: inkwell::support::LLVMString) -> Self {
    Self::LLVMError(format!("LLVM error: {}", value.to_string()))
  }
}

impl<C: Compiler> From<inkwell::builder::BuilderError> for Error<C> {
  fn from(value: inkwell::builder::BuilderError) -> Self {
    Self::LLVMError(format!("Builder error: {value}"))
  }
}

struct Compilation<'lazy, 'pool, 'llvm> {
  lazy: &'lazy gluezy::Lazy<'pool>,
  llvm: LLVMContext<'llvm>,
  functions: HashMap<
    gluezy::FunctionReference,
    inkwell::values::FunctionValue<'llvm>,
  >,
}

pub struct Program {
  context: inkwell::context::Context,
  global: gluezy::ModuleReference,
  cli_args: CliArgs,
}

pub struct ProgramCompilation<'ctx> {
  program: &'ctx Program,
  llvm: LLVMContext<'ctx>,
}

pub struct ProgramObjectFile {
  pub target: String,
  pub path: tempfile::TempPath,
}

impl<'lazy, 'pool, 'llvm> Compilation<'lazy, 'pool, 'llvm> {
  fn new(lazy: &'lazy gluezy::Lazy<'pool>, context: LLVMContext<'llvm>) -> Self {
    Self {
      lazy,
      llvm: context,
      functions: HashMap::new(),
    }
  }

  fn get_or_declare_function(&mut self,
    function: gluezy::FunctionReference,
  ) -> Result<LazyStructures, inkwell::values::FunctionValue<'llvm>> {
    // return it if we have it already
    if let Some(value) = self.functions.get(&function) {
      return Ok(*value);
    };

    // if we're still here, we need to make and store the function
    let name = {
      let borrow = function.rget_from(self.lazy);
      let name_id = borrow.header.name.id;
      self.lazy.pool.get(name_id)
    };

    let function_ty = types::make_function_type(self, function)?;

    // SPONGE: we need to determine this from the AST
    let linkage = Some(inkwell::module::Linkage::External);

    // add the function
    let function_value = self.llvm.module.add_function(&name, function_ty, linkage);

    // get an entry block
    let _entry = self.llvm.context.append_basic_block(function_value, "entry");

    // store to our cache
    self.functions.insert(function, function_value);

    Ok(function_value)
  }
}

impl Program {
  pub fn new(global: gluezy::ModuleReference, cli_args: CliArgs) -> Self {
    Self {
      context: inkwell::context::Context::create(),
      global,
      // SPONGE: need to parse this struct from settings
      cli_args,
    }
  }

  pub fn compile<'ctx>(&'ctx self, lazy: &gluezy::Lazy) -> Result<LazyStructures, ProgramCompilation<'ctx>> {
    let llvm_ctx = LLVMContext::new(&self.context, &self.cli_args);
    let mut comp = Compilation::new(lazy, llvm_ctx);

    compile::compile_module(&mut comp, self.global)?;

    Ok(ProgramCompilation {
      program: self,
      llvm: comp.llvm,
    })
  }
}

impl<'ctx> ProgramCompilation<'ctx> {
  pub fn save_to_file(self, file_type: FileType) -> Result<LazyStructures, ProgramObjectFile> {
    let file = {
      let temp_file_result = tempfile::Builder::new()
        .prefix("lazy-object-")
        .suffix(".o")
        .append(false)
        .tempfile();

      match temp_file_result {
        Ok(file) => file,
        Err(err) => todo!("tempfile err {err:?}"),
      }
    };

    let path = file.into_temp_path();

    self.llvm.module.verify()?;

    self.llvm.machine.write_to_file(
      &self.llvm.module,
      file_type,
      &path,
    )?;

    let target = self.llvm.machine.get_triple().as_str().to_string_lossy().to_string();

    Ok(ProgramObjectFile {
      path,
      target,
    })
  }

  pub fn dump(&self) -> String {
    self.llvm.dump_module()
  }

  pub fn optimize(&self, lazy: &gluezy::Lazy) -> Result<LazyStructures> {
    print_message!(lazy, {
      level: Info,
      force: false,
      description: line_dbg!("Optimizing LLVM code").into(),
      contents: MessageContents::None::<LazyStructures>,
    });

    self.llvm.run_passes(&self.program.cli_args.passes)
  }
}

impl ProgramObjectFile {
  pub fn link_with<'a>(self, out_path: &'a Path, linked: &[&str]) -> Result<LazyStructures, &'a Path> {
    let out_dir = if out_path.is_absolute() {
      out_path.parent()
        .expect("a parent directory in output")
        .to_owned()
    } else {
      std::env::current_dir().expect("cwd")
    };

    let path = self.path;

    // let output = out_path.file_name()
    //   .map(|path| path.to_str().expect("out_path to have a file name"))
    //   .unwrap_or("a.out");

    // SPONGE: may be possible to inject arguments this way?
    let links = linked.iter()
      .map(|name| format!("-l{name}"));

    let mut builder = cc::Build::new();

    let compiler = builder
      .object(&path)
      .out_dir(out_dir)
      .flags(links)
      .target(&self.target)
      .host(&self.target)
      .opt_level_str("2")
      .cargo_debug(false)
      .cargo_metadata(false)
      .cargo_output(false)
      .cargo_warnings(false)
      .get_compiler()
    ;

    let temp_path_str = path.to_str().expect("to convert path to str");
    let out_path_str = out_path.to_str().expect("to convert path to str");

    let compiler_result = compiler.to_command()
      .args(compiler.args())
      .args(["-o", out_path_str])
      .arg(temp_path_str)
      .spawn()
      .expect("to spawn compiler subprocess")
      .wait()
      .expect("to wait for compiler subproccess")
    ;

    assert!(compiler_result.success(), "compiler subprocess errored");
    assert!(out_path.exists(), "didn't create output file");

    path.close().expect("to close temp file");

    Ok(out_path)
  }
}

impl<C: Compiler> From<crate::Error<C>> for log::PrintableMessage<C> {
  fn from(value: crate::Error<C>) -> Self {
    match value {
      crate::Error::StillUnresolved { what, note, span } => Self {
        level: Level::Error,
        force: true,
        description: format!("unresolved in generation: {what}"),
        contents: MessageContents::WithinSource(WithinSource::new(
          vec![MessageSection {
            text: note,
            span,
          }],
        )),
      },
      crate::Error::LLVMError(description) => Self {
        level: Level::Error,
        force: true,
        description,
        contents: MessageContents::None,
      },
    }
  }
}
