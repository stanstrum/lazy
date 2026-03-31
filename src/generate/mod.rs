use crate::lang;

#[derive(Debug)]
pub enum Error {

}

type Result<T = ()> = std::result::Result<T, Error>;

struct LLVMContext<'ctx> {
  context: &'ctx inkwell::context::Context,
  builder: inkwell::builder::Builder<'ctx>,
  module: inkwell::module::Module<'ctx>,
  machine: inkwell::targets::TargetMachine,
  // sym_table: RefCell<HashMap<String, PointerValue<'ctx>>>,
}

// SPONGE
struct CliArgs {
  target: Option<String>,
  opt_level: OptimizationLevel,
  passes: String,
}

#[derive(Clone, Copy)]
enum OptimizationLevel {
  O0,
  O1,
  O2,
  O3,
}

impl From<OptimizationLevel> for inkwell::OptimizationLevel {
  fn from(value: OptimizationLevel) -> Self {
    match value {
      OptimizationLevel::O0 => Self::None,
      OptimizationLevel::O1 => Self::Less,
      OptimizationLevel::O2 => Self::Default,
      OptimizationLevel::O3 => Self::Aggressive,
    }
  }
}

// SPONGE
/// from <https://github.com/acolite-d/llvm-tutorial-in-rust-using-inkwell>
impl<'ctx> LLVMContext<'ctx> {
  fn new(context: &'ctx inkwell::context::Context, cli_args: &CliArgs) -> Self {
    let builder = context.create_builder();
    let module = context.create_module("kaleidrs_module");

    let triple = match cli_args.target.as_ref() {
      None => inkwell::targets::TargetMachine::get_default_triple(),
      Some(target_str) => inkwell::targets::TargetTriple::create(target_str.as_str()),
    };

    let config = Default::default();
    let sponge = inkwell::targets::Target::initialize_native(&config)
      .expect("initialize native");

    // SPONGE: what to do here?  do i need to parse the triple to figure out
    //         what target i need to initialize, or do i just plain initialize
    //         everything every time?
    //
    // let sponge = inkwell::targets::Target::initialize_all(&config);

    let target = inkwell::targets::Target::from_triple(&triple)
      .expect("Unknown target: please specify a target");

    let machine = target
      .create_target_machine(
        &triple,
        "generic",
        "",
        cli_args.opt_level.into(),
        inkwell::targets::RelocMode::Default,
        inkwell::targets::CodeModel::Default,
      )
      .unwrap();

    Self {
      context,
      builder,
      module,
      machine,
      // sym_table: RefCell::new(HashMap::new()),
    }
  }

  pub fn dump_module(&self) {
    println!(
      "LLVM IR Representation:\n{}",
      self.module.print_to_string().to_string(),
    );
  }

  /// This method will write assembly of module to memory buffer, read as UTF-8 and print
  /// to screen.
  pub fn dump_assembly(&self) -> () {
    let buf = self.machine
      .write_to_memory_buffer(&self.module, inkwell::targets::FileType::Assembly)
      .expect("Failed to write assembly representation");

    println!(
      "Assembly Representation:\n{}\n",
      std::str::from_utf8(buf.as_slice()).unwrap()
    );
  }

  /// Optimization passes
  pub fn run_passes(&self, passes: &str) {
    if !passes.is_empty() {
      let pass_options = inkwell::passes::PassBuilderOptions::create();

      // Default passes
      pass_options.set_verify_each(true);
      pass_options.set_debug_logging(false);

      self.module
        .run_passes(passes, &self.machine, pass_options)
        .unwrap();
    }
  }
}

fn compile_module(lazy: &lang::Lazy, ctx: &LLVMContext, module: lang::reference::ModuleReference) -> Result {
  todo!()
}

pub fn compile(lazy: &lang::Lazy, global: lang::reference::ModuleReference) -> Result {
  let cli_args = CliArgs {
    target: None,
    opt_level: OptimizationLevel::O0,
    passes: "instcombine,reassociate,gvn,simplifycfg,mem2reg".into(),
  };

  let ctx = inkwell::context::Context::create();
  let llvm_ctx = LLVMContext::new(&ctx, &cli_args);

  compile_module(lazy, &llvm_ctx, global)?;

  llvm_ctx.run_passes(&cli_args.passes);

  todo!()
}
