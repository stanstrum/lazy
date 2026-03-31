mod context;
mod args;
mod compile;

mod types;

use std::collections::HashMap;

use crate::lang;
use crate::lang::span::GetSpan;
use crate::lang::reference::{Reference, Store};
use crate::resolve::TypeOf;
use crate::tokenize::token;

use {args::*, context::*};

#[derive(Debug)]
pub enum Error {
  StillUnresolved {
    what: String,
    note: String,
    span: token::Span,
  },
}

type Result<T = ()> = std::result::Result<T, Error>;

struct Compilation<'lazy, 'pool, 'llvm> {
  lazy: &'lazy lang::Lazy<'pool>,
  llvm: LLVMContext<'llvm>,
  functions: HashMap<
    lang::reference::FunctionReference,
    inkwell::values::FunctionValue<'llvm>,
  >,
}

impl<'lazy, 'pool, 'llvm> Compilation<'lazy, 'pool, 'llvm> {
  fn new(lazy: &'lazy lang::Lazy<'pool>, context: LLVMContext<'llvm>) -> Self {
    Self {
      lazy,
      llvm: context,
      functions: HashMap::new(),
    }
  }

  fn get_or_declare_function(&mut self,
    function: lang::reference::FunctionReference,
  ) -> Result<inkwell::values::FunctionValue<'llvm>> {
    // return it if we have it already
    if let Some(value) = self.functions.get(&function) {
      return Ok(*value);
    };

    // if we're still here, we need to make and store the function
    let name = {
      let borrow = function.rget_from(self.lazy);
      let name_id = borrow.header.name.id;
      self.lazy.pool.get(name_id).collect::<String>()
    };

    let function_ty = types::make_function_type(self, function)?;

    // SPONGE: we need to determine this from the AST
    let linkage = Some(inkwell::module::Linkage::External);

    // add the function
    let function_value = self.llvm.module.add_function(&name, function_ty, linkage);

    // store to our cache
    self.functions.insert(function, function_value);

    Ok(function_value)
  }
}

pub fn compile(lazy: &lang::Lazy, global: lang::reference::ModuleReference) -> Result {
  // SPONGE: parse CliArgs from settings
  let cli_args = CliArgs {
    target: None,
    opt_level: OptimizationLevel::O0,
    passes: "instcombine,reassociate,gvn,simplifycfg,mem2reg".into(),
  };

  // initialize contexts
  let ctx = inkwell::context::Context::create();
  let llvm_ctx = LLVMContext::new(&ctx, &cli_args);

  let mut comp = Compilation::new(lazy, llvm_ctx);

  // compile
  compile::compile_module(&mut comp, global)?;

  // debug
  comp.llvm.dump_module();

  // optimize
  comp.llvm.run_passes(&cli_args.passes);

  todo!()
}
