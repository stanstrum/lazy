mod error;

use std::process::Command;
use std::rc::Rc;
use std::cell::RefCell;

use inkwell::{
  builder::Builder,
  context::Context,
  module::{
    Linkage,
    Module,
  },
  types::{
    ArrayType,
    BasicMetadataTypeEnum,
    BasicType,
    BasicTypeEnum,
    FunctionType,
    PointerType,
    VoidType,
  },
  values::{
    BasicValue,
    BasicValueEnum,
    FunctionValue,
    PointerValue,
  },
  AddressSpace
};

use crate::tokenizer::{
  self,
  GetSpan
};

use crate::typechecker::{
  lang,
  lang::intrinsics::{
    self,
    Intrinsic,
  },
  Domain,
  DomainMember,
  Program,
  TypeOf,
};

pub(crate) use error::*;

#[allow(unused)]
struct GeneratorScope<'a> {
  scope: Rc<RefCell<lang::VariableScope>>,
  pointers: Option<Vec<PointerValue<'a>>>,
}

#[allow(unused)]
pub(crate) struct Generator<'a> {
  context: &'a Context,
  module: Module<'a>,
  builder: &'a Builder<'a>,
  scopes: Vec<GeneratorScope<'a>>,
}

trait Generate<'a> {
  type Out;

  fn generate(&self, generator: &mut Generator<'a>) -> Result<Self::Out, GeneratorError>;
}

enum GeneratorType<'a> {
  Basic(BasicTypeEnum<'a>),
  Void(VoidType<'a>),
  Function(FunctionType<'a>),
}

impl<'a> GeneratorType<'a> {
  fn into_function(self) -> FunctionType<'a> {
    let Self::Function(function) = self else {
      panic!("bad cast");
    };

    function
  }

  fn into_basic(self) -> BasicTypeEnum<'a> {
    let Self::Basic(basic) = self else {
      panic!("bad cast");
    };

    basic
  }

  fn fn_type(&self, param_types: &[BasicMetadataTypeEnum<'a>], is_var_args: bool) -> FunctionType<'a> {
    match self {
      GeneratorType::Basic(basic) => basic.fn_type(param_types, is_var_args),
      GeneratorType::Void(void) => void.fn_type(param_types, is_var_args),
      _ => panic!("bad cast"),
    }
  }

  fn ptr_type(&self, address_space: AddressSpace) -> PointerType<'a> {
    match self {
      GeneratorType::Basic(basic) => basic.ptr_type(address_space),
      GeneratorType::Function(func) => func.ptr_type(address_space),
      _ => panic!("bad cast"),
    }
  }

  fn array_type(&self, size: u32) -> ArrayType<'a> {
    match self {
      GeneratorType::Basic(basic) => basic.array_type(size),
      _ => panic!("bad cast"),
    }
  }

  fn into_basic_metadata(self) -> BasicMetadataTypeEnum<'a> {
    match self.into_basic() {
      BasicTypeEnum::ArrayType(ty) => BasicMetadataTypeEnum::ArrayType(ty),
      BasicTypeEnum::FloatType(ty) => BasicMetadataTypeEnum::FloatType(ty),
      BasicTypeEnum::IntType(ty) => BasicMetadataTypeEnum::IntType(ty),
      BasicTypeEnum::PointerType(ty) => BasicMetadataTypeEnum::PointerType(ty),
      BasicTypeEnum::StructType(ty) => BasicMetadataTypeEnum::StructType(ty),
      BasicTypeEnum::VectorType(ty) => BasicMetadataTypeEnum::VectorType(ty),
    }
  }
}

impl<'a> Generate<'a> for Intrinsic {
  type Out = GeneratorType<'a>;

  fn generate(&self, generator: &mut Generator<'a>) -> Result<Self::Out, GeneratorError> {
    Ok(match self {
      Intrinsic::Void => GeneratorType::Void(generator.context.void_type()),
      | Intrinsic::U8
      | Intrinsic::I8 => GeneratorType::Basic(BasicTypeEnum::IntType(generator.context.i8_type())),
      | Intrinsic::U16
      | Intrinsic::I16 => GeneratorType::Basic(BasicTypeEnum::IntType(generator.context.i16_type())),
      | Intrinsic::U32
      | Intrinsic::I32 => GeneratorType::Basic(BasicTypeEnum::IntType(generator.context.i32_type())),
      | Intrinsic::U64
      | Intrinsic::I64 => GeneratorType::Basic(BasicTypeEnum::IntType(generator.context.i64_type())),
      Intrinsic::F32 => todo!(),
      Intrinsic::F64 => todo!(),
    })
  }
}

trait ResolveToU32 {
  fn resolve_to_u32(&self) -> u32;
}

impl ResolveToU32 for tokenizer::Literal {
  fn resolve_to_u32(&self) -> u32 {
    match &self.kind {
      tokenizer::LiteralKind::Integer(integer) => *integer as u32,
      tokenizer::LiteralKind::FloatingPoint(_) => todo!(),
      tokenizer::LiteralKind::UnicodeString(_) => todo!(),
      tokenizer::LiteralKind::CString(_) => todo!(),
      tokenizer::LiteralKind::ByteString(_) => todo!(),
      tokenizer::LiteralKind::UnicodeChar(_) => todo!(),
      tokenizer::LiteralKind::ByteChar(_) => todo!(),
    }
  }
}

impl ResolveToU32 for lang::Instruction {
  fn resolve_to_u32(&self) -> u32 {
    match self {
      | lang::Instruction::Assign { .. }
      | lang::Instruction::Return { .. } => panic!("can't resolve to u32"),
      lang::Instruction::Call { .. } => todo!(),
      lang::Instruction::Value(value) => value.resolve_to_u32(),
    }
  }
}

impl ResolveToU32 for lang::Value {
  fn resolve_to_u32(&self) -> u32 {
    match self {
      lang::Value::Variable(_) => todo!(),
      lang::Value::Instruction(instruction) => instruction.resolve_to_u32(),
      lang::Value::Literal { literal, .. } => literal.resolve_to_u32(),
    }
  }
}

impl<'a> Generate<'a> for lang::Type {
  type Out = GeneratorType<'a>;

  fn generate(&self, generator: &mut Generator<'a>) -> Result<Self::Out, GeneratorError> {
    match self {
      Self::Function { args, return_ty, .. } => {
        let args = args.iter()
          .map(|ty| {
            Ok(
              ty.generate(generator)?
                .into_basic_metadata()
            )
          })
          .collect::<Result<Vec<_>, _>>()?;

        Ok(GeneratorType::Function(
          return_ty.generate(generator)?
            .fn_type(&args, false)
        ))
      },
      Self::Intrinsic { kind, .. } => kind.generate(generator),
      Self::FuzzyInteger { .. } => intrinsics::USIZE.generate(generator),
      Self::FuzzyString { size, element_ty, .. } => {
        Ok(GeneratorType::Basic(
          element_ty.generate(generator)?
            .array_type(*size as u32)
            .as_basic_type_enum()
            .ptr_type(Default::default())
            .as_basic_type_enum()
        ))
      },
      Self::ReferenceTo { ty, .. } => {
        Ok(GeneratorType::Basic(
          ty.generate(generator)?
            .ptr_type(Default::default())
            .as_basic_type_enum()
        ))
      },
      Self::SizedArrayOf { count, ty, .. } => {
        Ok(GeneratorType::Basic(
          ty.generate(generator)?.array_type(
            count.resolve_to_u32()
          ).as_basic_type_enum()
        ))
      },
      _ => UnresolvedSnafu {
        span: self.get_span(),
      }.fail()
    }
  }
}

impl<'a> Generate<'a> for lang::TypeCell {
  type Out = <lang::Type as Generate<'a>>::Out;

  fn generate(&self, generator: &mut Generator<'a>) -> Result<Self::Out, GeneratorError> {
    self.borrow().generate(generator)
  }
}

impl<'a> Generate<'a> for lang::VariableReference {
  type Out = BasicValueEnum<'a>;

  fn generate(&self, _generator: &mut Generator<'a>) -> Result<Self::Out, GeneratorError> {
    todo!()
  }
}

impl<'a> Generate<'a> for lang::Instruction {
  type Out = Option<BasicValueEnum<'a>>;

  fn generate(&self, generator: &mut Generator<'a>) -> Result<Self::Out, GeneratorError> {
    Ok(match self {
      lang::Instruction::Assign { .. } => None,
      lang::Instruction::Call { .. } => todo!(),
      lang::Instruction::Return { .. } => None,
      lang::Instruction::Value(value) => value.generate(generator)?,
    })
  }
}

enum StringGenerationKind {
  GlobalReference,
  StackAllocated,
}

fn determine_string_type(ty: lang::Type) -> StringGenerationKind {
  match ty {
    lang::Type::SizedArrayOf { .. } => {
      StringGenerationKind::StackAllocated
    },
    | lang::Type::FuzzyString { .. }
    | lang::Type::ReferenceTo { .. } => {
      StringGenerationKind::GlobalReference
    },
    invalid => unreachable!("{invalid:?}"),
  }
}

fn generate_string_literal<'a>(generator: &mut Generator<'a>, ty: lang::Type, element_ty: intrinsics::Intrinsic, null_terminator: bool, content: &str) -> BasicValueEnum<'a> {
  let element_ty = element_ty.generate(generator).unwrap().into_basic().into_int_type();

  let mut values = content.chars()
    .map(|ch| {
      element_ty.const_int(ch as u64, false)
    })
    .collect::<Vec<_>>();

  if null_terminator {
    values.push(element_ty.const_zero());
  };

  match determine_string_type(ty) {
    StringGenerationKind::StackAllocated => {
      element_ty.const_array(&values).as_basic_value_enum()
    },
    StringGenerationKind::GlobalReference => {
      let value = element_ty.const_array(&values).as_basic_value_enum();

      let global = generator.module.add_global(
        value.get_type().as_basic_type_enum(),
        Default::default(),
        "unicode_string"
      );

      global.set_initializer(&value);

      global.as_basic_value_enum()
    },
  }
}

impl<'a> Generate<'a> for lang::Value {
  type Out = Option<BasicValueEnum<'a>>;

  fn generate(&self, generator: &mut Generator<'a>) -> Result<Self::Out, GeneratorError> {
    Ok(match self {
      lang::Value::Variable(variable) => Some(variable.generate(generator)?),
      lang::Value::Instruction(instruction) => instruction.generate(generator)?,
      lang::Value::Literal { literal, ty } => {
        match &literal.kind {
          tokenizer::LiteralKind::Integer(integer) => Some(
            ty
              .generate(generator)?
              .into_basic()
              .into_int_type()
              .const_int(*integer, false)
              .as_basic_value_enum()
          ),
          tokenizer::LiteralKind::FloatingPoint(float) => Some(
            ty
              .generate(generator)?
              .into_basic()
              .into_float_type()
              .const_float(*float)
              .as_basic_value_enum()
          ),
          tokenizer::LiteralKind::UnicodeString(content) => Some(generate_string_literal(
            generator,
            self.type_of().unwrap(),
            intrinsics::UNICODE_CHAR,
            false,
            content
          )),
          tokenizer::LiteralKind::CString(content) => Some(generate_string_literal(
            generator,
            self.type_of().unwrap(),
            intrinsics::C_CHAR,
            true,
            content
          )),
          tokenizer::LiteralKind::ByteString(content) => Some(generate_string_literal(
            generator,
            self.type_of().unwrap(),
            Intrinsic::U8,
            false,
            content
          )),
          tokenizer::LiteralKind::UnicodeChar(ch) => Some({
            intrinsics::UNICODE_CHAR.generate(generator)?
              .into_basic()
              .into_int_type()
              .const_int(*ch as u64, false)
              .as_basic_value_enum()
          }),
          tokenizer::LiteralKind::ByteChar(ch) => Some({
            generator.context.i8_type()
              .const_int(*ch as u64, false)
              .as_basic_value_enum()
          }),
        }
      },
    })
  }
}

impl<'a> Generator<'a> {
  pub(crate) fn new(context: &'a Context, builder: &'a Builder<'a>) -> Self {
    Self {
      context,
      module: context.create_module("program"),
      builder,
      scopes: vec![],
    }
  }

  // TODO: better error handling
  pub(crate) fn create_binary_executable(&self, executable_path: &str) -> Result<(), GeneratorError> {
    println!("Verifying ...");

    if let Err(error) = self.module.verify() {
      println!("LLVM verification failed: {}", error.to_string_lossy());
      println!("Source:\n{}", self.module.print_to_string().to_string_lossy());

      panic!("verification failed");
    };

    println!("Writing LLVM to a.ll ...");

    let cwd = std::env::current_dir().expect("couldn't get working dir");
    let out_file = cwd.join("a.ll");

    self.module
      .print_to_file(out_file.to_str().unwrap())
      .expect("error printing to file");

    // compile llvm code
    let exit_status = {
      println!("Running `llc` ...");
      Command::new("llc")
        // this argument is surprisingly important
        .arg("--relocation-model=pic")
        .args(["-o", "a.s"])
        .arg("a.ll")
        .stdout(std::process::Stdio::piped())
        .spawn().unwrap()
        .wait()
        .expect("error compiling emitted llvm code")
    };

    if !exit_status.success() {
      panic!("llc failed");
    };

    // assemble `llc` output
    let exit_status = {
      println!("Running `as` ...");

      Command::new("as")
        .args(["-o", "a.o"])
        .arg("a.s")
        .stdout(std::process::Stdio::piped())
        .spawn().unwrap()
        .wait()
        .expect("error assembling emitted assembly code")
    };

    if !exit_status.success() {
      panic!("as failed");
    };

    // link `as` output
    let exit_status = {
      println!("Running `cc` ...");
      Command::new("cc")
        .args(["-o", executable_path])
        .arg("a.o")
        .stdout(std::process::Stdio::piped())
        .spawn().unwrap()
        .wait()
        .expect("error linking emitted object code")
    };

    if !exit_status.success() {
      panic!("cc failed");
    };

    println!("Your shiny new Lazy program is located in `./program`.  Enjoy!");

    Ok(())
  }

  fn declare_function(&mut self, func: &lang::Function) -> Result<FunctionValue<'a>, GeneratorError> {
    // generate func type so we can add it to the module
    let ty = func.type_of()
      .expect("couldn't get type of function")
      .generate(self)?
      .into_function();

    Ok(
      self.module.add_function(&func.name, ty, Some(Linkage::Internal))
    )
  }

  fn register_scope(&mut self, scope: &Rc<RefCell<lang::VariableScope>>) -> usize {
    let mut borrowed = scope.borrow_mut();

    if borrowed.generator_id.is_none() {
      let id = self.scopes.len();

      self.scopes.push(GeneratorScope {
        scope: scope.to_owned(),
        pointers: None,
      });

      borrowed.generator_id = Some(id);
    };

    borrowed.generator_id.unwrap()
  }

  fn get_variable_reference(&mut self, reference: &lang::VariableReference) -> Result<PointerValue<'a>, GeneratorError> {
    let id = reference.scope.borrow()
      .generator_id
      .expect("target scope has no generator id");

    let scope = self.scopes.get(id).unwrap();

    Ok(
      scope.pointers
        .as_ref()
        .expect("target scope has no pointers")
        .get(reference.id)
        .unwrap()
        .to_owned()
    )
  }

  fn resolve_dest(&mut self, value: &lang::Value) -> Result<PointerValue<'a>, GeneratorError> {
    match value {
      lang::Value::Variable(var) => {
        self.get_variable_reference(var)
      },
      lang::Value::Instruction(_) => todo!(),
      lang::Value::Literal { .. } => todo!(),
    }
  }

  fn generate_block(&mut self, block: &mut lang::Block, _function: FunctionValue<'a>) -> Result<(), GeneratorError> {
    let scope_id = self.register_scope(&block.variables);

    let mut pointers = vec![];
    for variable in block.variables.borrow().inner.iter() {
      let ty = variable.ty.generate(self)?.into_basic();
      let pointer = self.builder.build_alloca(ty, &variable.name);

      pointers.push(pointer);
    };

    self.scopes.get_mut(scope_id).unwrap().pointers = Some(pointers);

    for inst in block.body.iter() {
      match inst {
        lang::Instruction::Assign { dest, value, .. } => {
          let dest = self.resolve_dest(dest)?;
          let value = value.generate(self)?
            .unwrap();

          self.builder.build_store(dest, value);
        },
        lang::Instruction::Call { .. } => todo!(),
        lang::Instruction::Return { value, .. } => {
          if let Some(value) = &value {
            self.builder.build_return(
              // intentional -- we should error here if there's no value
              // generated, as we expect one
              Some(
                &value.generate(self)?.unwrap()
              )
            );
          } else {
            self.builder.build_return(None);
          };
        },
        lang::Instruction::Value(_) => todo!(),
      }
    };

    Ok(())
  }

  fn generate_function(&mut self, func: &mut lang::Function, value: FunctionValue<'a>) -> Result<(), GeneratorError> {
    let basic_block = self.context.append_basic_block(value, "entry");
    self.builder.position_at_end(basic_block);

    // let arguments = func.arguments.inner.borrow()
    //   .iter()
    //   .map(|arg| Ok(
    //     arg.ty.generate(self)?
    //       .into_basic()
    //   ))
    //   .collect::<Result<Vec<_>, _>>()?;

    self.generate_block(&mut func.body, value)
  }

  fn generate_domain(&mut self, domain: &mut Domain) -> Result<(), GeneratorError> {
    let mut funcs = vec![];

    for member in domain.inner.values_mut() {
      match member {
        DomainMember::Domain(domain) => self.generate_domain(domain)?,
        DomainMember::Function(func) => {
          funcs.push(
            self.declare_function(func)?
          );
        },
        DomainMember::Type(_) => {
          dbg!("type ignored");
        },
      };
    };

    for (func, value) in domain.inner.values_mut()
      .filter_map(|member| {
        if let DomainMember::Function(func) = member {
          Some(func)
        } else {
          None
        }
      }).zip(funcs)
    {
      self.generate_function(func, value)?;
    };

    Ok(())
  }

  pub(crate) fn generate(&mut self, program: &mut Program) -> Result<(), GeneratorError> {
    for data in program.inner.values_mut() {
      println!("Generating {:?}", data.path.to_string_lossy());

      self.generate_domain(&mut data.domain)?;
    };

    Ok(())
  }
}
