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
    AnyTypeEnum,
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
  AddressSpace,
};

use crate::tokenizer::{
  self,
  GetSpan,
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
  pointers: Option<Vec<BasicValueEnum<'a>>>,
}

#[allow(unused)]
pub(crate) struct Generator<'a> {
  context: &'a Context,
  module: Module<'a>,
  builder: &'a Builder<'a>,
  scopes: Vec<GeneratorScope<'a>>,
  current_function: Option<FunctionValue<'a>>,
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

impl<'a> From<AnyTypeEnum<'a>> for GeneratorType<'a> {
  fn from(value: AnyTypeEnum<'a>) -> Self {
    match value {
      AnyTypeEnum::ArrayType(ty) => Self::Basic(BasicTypeEnum::ArrayType(ty)),
      AnyTypeEnum::FloatType(ty) => Self::Basic(BasicTypeEnum::FloatType(ty)),
      AnyTypeEnum::FunctionType(ty) => Self::Function(ty),
      AnyTypeEnum::IntType(ty) => Self::Basic(BasicTypeEnum::IntType(ty)),
      AnyTypeEnum::PointerType(ty) => Self::Basic(BasicTypeEnum::PointerType(ty)),
      AnyTypeEnum::StructType(ty) => Self::Basic(BasicTypeEnum::StructType(ty)),
      AnyTypeEnum::VectorType(ty) => Self::Basic(BasicTypeEnum::VectorType(ty)),
      AnyTypeEnum::VoidType(ty) => Self::Void(ty),
    }
  }
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
      Intrinsic::Bool => GeneratorType::Basic(BasicTypeEnum::IntType(generator.context.bool_type())),
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

pub(crate) trait ResolveToU32 {
  fn resolve_to_u32(&self) -> u32;
}

impl ResolveToU32 for tokenizer::Literal {
  fn resolve_to_u32(&self) -> u32 {
    match &self.kind {
      tokenizer::LiteralKind::Integer(integer) => *integer as u32,
      | tokenizer::LiteralKind::FloatingPoint(_)
      | tokenizer::LiteralKind::UnicodeString(_)
      | tokenizer::LiteralKind::CString(_)
      | tokenizer::LiteralKind::ByteString(_)
      | tokenizer::LiteralKind::UnicodeChar(_)
      | tokenizer::LiteralKind::ByteChar(_)
      | tokenizer::LiteralKind::Boolean(_) => todo!(),
    }
  }
}

impl ResolveToU32 for lang::Instruction {
  fn resolve_to_u32(&self) -> u32 {
    match self {
      | lang::Instruction::Assign { .. }
      | lang::Instruction::Return { .. } => panic!("can't resolve to u32"),
      lang::Instruction::Call { .. } => todo!(),
      lang::Instruction::Block(_) => todo!(),
      lang::Instruction::Value(value) => value.resolve_to_u32(),
      lang::Instruction::ControlFlow(_) => todo!(),
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
      Self::UnsizedArrayOf { ty, .. } => {
        ty.generate(generator)
      },
      _ => {
        dbg!(self);

        UnresolvedSnafu {
          span: self.get_span(),
        }.fail()
      },
    }
  }
}

impl<'a> Generate<'a> for lang::TypeCell {
  type Out = <lang::Type as Generate<'a>>::Out;

  fn generate(&self, generator: &mut Generator<'a>) -> Result<Self::Out, GeneratorError> {
    self.borrow().generate(generator)
  }
}

impl<'a> Generate<'a> for lang::ControlFlow {
  type Out = Option<BasicValueEnum<'a>>;

  fn generate(&self, generator: &mut Generator<'a>) -> Result<Self::Out, GeneratorError> {
    match self {
      lang::ControlFlow::While { condition, body, .. } => {
        let while_outer_block = generator.context.append_basic_block(
          generator.current_function.unwrap(),
          "while_body_outer"
        );

        let while_inner_block = generator.context.append_basic_block(
          generator.current_function.unwrap(),
          "while_body_inner"
        );

        let while_break_block = generator.context.append_basic_block(
          generator.current_function.unwrap(),
          "while_break"
        );

        generator.builder.build_unconditional_branch(while_outer_block);
        generator.builder.position_at_end(while_outer_block);

        let condition = condition.generate(generator)?
          .expect("comparison condition must yield a value")
          .as_basic_value_enum()
          .into_int_value();

        generator.builder.build_conditional_branch(
          condition,
          while_inner_block,
          while_break_block
        );

        generator.builder.position_at_end(while_inner_block);

        body.generate(generator)?;

        generator.builder.build_unconditional_branch(while_outer_block);
        generator.builder.position_at_end(while_break_block);

        // TODO: figure out how to return a value using a break
        Ok(None)
      },
      lang::ControlFlow::DoWhile { condition, body, .. } => {
        let do_while_inner = generator.context.append_basic_block(
          generator.current_function.unwrap(),
          "do_while_inner"
        );

        let do_while_break = generator.context.append_basic_block(
          generator.current_function.unwrap(),
          "do_while_break"
        );

        generator.builder.build_unconditional_branch(do_while_inner);
        generator.builder.position_at_end(do_while_inner);

        body.generate(generator)?;

        let condition = condition.generate(generator)?
          .expect("comparison condition must yield a value")
          .as_basic_value_enum()
          .into_int_value();

        generator.builder.build_conditional_branch(condition, do_while_inner, do_while_break);

        generator.builder.position_at_end(do_while_break);

        // TODO: figure out how to return a value using a break
        Ok(None)
      },
      lang::ControlFlow::Until { condition, body, .. } => {
        let until_outer_block = generator.context.append_basic_block(
          generator.current_function.unwrap(),
          "until_body_outer"
        );

        let until_inner_block = generator.context.append_basic_block(
          generator.current_function.unwrap(),
          "until_body_inner"
        );

        let until_break_block = generator.context.append_basic_block(
          generator.current_function.unwrap(),
          "until_break"
        );

        generator.builder.build_unconditional_branch(until_outer_block);
        generator.builder.position_at_end(until_outer_block);

        let not_condition = condition.generate(generator)?
          .expect("comparison condition must yield a value")
          .as_basic_value_enum()
          .into_int_value();

        let condition = generator.builder.build_not(not_condition, "until_not_condition");

        generator.builder.build_conditional_branch(
          condition,
          until_inner_block,
          until_break_block
        );

        generator.builder.position_at_end(until_inner_block);

        body.generate(generator)?;

        generator.builder.build_unconditional_branch(until_outer_block);
        generator.builder.position_at_end(until_break_block);

        // TODO: figure out how to return a value using a break
        Ok(None)
      },
      lang::ControlFlow::DoUntil { .. } => todo!(),
      | lang::ControlFlow::If { .. }
      | lang::ControlFlow::For { .. }
      | lang::ControlFlow::Loop { .. } => todo!(),
    }
  }
}


impl<'a> Generate<'a> for lang::Instruction {
  type Out = Option<BasicValueEnum<'a>>;

  fn generate(&self, generator: &mut Generator<'a>) -> Result<Self::Out, GeneratorError> {
    Ok({
      match self {
        lang::Instruction::Assign { dest, value, .. } => {
          let dest = generator.resolve_dest(dest)?;
          let value = value.generate(generator)?
            .unwrap();

          generator.builder.build_store(
            dest,
            generator.builder.build_bitcast(
              value,
              GeneratorType::from(dest.get_type().get_element_type()).into_basic(),
              "store_bitcast"
            )
          );

          None
        },
        lang::Instruction::Call { .. } => todo!(),
        lang::Instruction::Return { value, .. } => {
          if let Some(value) = &value {
            generator.builder.build_return(
              // intentional -- we should error here if there's no value
              // generated, as we expect one
              Some(
                &value.generate(generator)?.unwrap()
              )
            );
          } else {
            generator.builder.build_return(None);
          };

          None
        },
        lang::Instruction::Value(value) => value.generate(generator)?,
        lang::Instruction::Block(block) => block.generate(generator)?,
        lang::Instruction::ControlFlow(ctrl_flow) => ctrl_flow.generate(generator)?,
      }
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
      lang::Value::Variable(reference) => Some(generator.load_variable_reference(reference)?),
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
          tokenizer::LiteralKind::Boolean(boolean) => Some({
            generator.context.bool_type()
              .const_int(*boolean as u64, false)
              .as_basic_value_enum()
          })
        }
      },
    })
  }
}

impl<'a> Generate<'a> for lang::Block {
  type Out = Option<BasicValueEnum<'a>>;

  fn generate(&self, generator: &mut Generator<'a>) -> Result<Self::Out, GeneratorError> {
    let scope_id = generator.register_scope(&self.variables);

    let mut pointers = vec![];
    for variable in self.variables.borrow().inner.iter() {
      let ty = variable.ty.generate(generator)?.into_basic();
      let pointer = generator.builder.build_alloca(ty, &variable.name);

      pointers.push(pointer.as_basic_value_enum());
    };

    generator.scopes.get_mut(scope_id).unwrap().pointers = Some(pointers);

    let (last, instructions) = if self.returns_last {
      let (last, instructions) = self.body.split_last().unwrap();

      (Some(last), instructions)
    } else {
      (None, self.body.as_slice())
    };

    for inst in instructions {
      inst.generate(generator)?;
    };

    if let Some(last) = last {
      last.generate(generator)
    } else {
      Ok(None)
    }
  }
}

impl<'a> Generate<'a> for lang::ExternFunction {
  type Out = FunctionValue<'a>;

  fn generate(&self, generator: &mut Generator<'a>) -> Result<Self::Out, GeneratorError> {
    let param_types = self.arguments.borrow().inner.iter()
      .map(|argument| Ok(
        argument.ty.generate(generator)?
          .into_basic_metadata()
      ))
      .collect::<Result<Vec<_>, _>>()?;

    let ty = self.return_ty.generate(generator)?
      .fn_type(&param_types, self.variadic);

    Ok(generator.module.add_function(&self.name, ty, Some(Linkage::External)))
  }
}

impl<'a> Generator<'a> {
  pub(crate) fn new(context: &'a Context, builder: &'a Builder<'a>) -> Self {
    Self {
      context,
      module: context.create_module("program"),
      builder,
      scopes: vec![],
      current_function: None,
    }
  }

  // TODO: better error handling
  pub(crate) fn create_binary_executable(&self, executable_path: &str) -> Result<(), GeneratorError> {
    println!("Verifying ...");

    if let Err(error) = self.module.verify() {
      println!("LLVM verification failed: {}", error.to_string_lossy());
      println!("Source:\n{}", self.module.print_to_string().to_string_lossy());

      println!("verification failed");
    };

    if self.module.get_function("main").is_none() {
      panic!("no main function");
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
      self.module.add_function(&func.name, ty, None)
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

  fn load_variable_reference(&mut self, reference: &lang::VariableReference) -> Result<BasicValueEnum<'a>, GeneratorError> {
    let kind = { reference.get().kind };

    Ok(match kind {
      lang::VariableKind::LocalVariable => {
        self.builder.build_load(
          self.get_variable_reference_pointer(reference)?,
          &format!("load_{}", &reference.get().name)
        )
      },
      lang::VariableKind::Argument => {
        let id = reference.scope.borrow()
          .generator_id
          .expect("target scope has no generator id");

        self.scopes[id].pointers
          .as_ref()
          .expect("target scope has no pointers")
          .get(reference.id)
          .unwrap()
          .as_basic_value_enum()
      },
    })
  }

  fn get_variable_reference_pointer(&mut self, reference: &lang::VariableReference) -> Result<PointerValue<'a>, GeneratorError> {
    let id = reference.scope.borrow()
      .generator_id
      .expect("target scope has no generator id");

    let scope = self.scopes.get(id).unwrap();

    Ok(
      match &reference.get().kind {
        lang::VariableKind::LocalVariable => {
          scope.pointers
          .as_ref()
          .expect("target scope has no pointers")
          .get(reference.id)
          .unwrap()
          .into_pointer_value()
          .to_owned()
        },
        lang::VariableKind::Argument => panic!("can't assign to a parameter"),
      }
    )
  }

  fn resolve_dest(&mut self, value: &lang::Value) -> Result<PointerValue<'a>, GeneratorError> {
    match value {
      lang::Value::Variable(var) => {
        self.get_variable_reference_pointer(var)
      },
      lang::Value::Instruction(_) => todo!(),
      lang::Value::Literal { .. } => todo!(),
    }
  }

  fn generate_function(&mut self, func: &mut lang::Function, value: FunctionValue<'a>) -> Result<(), GeneratorError> {
    let basic_block = self.context.append_basic_block(value, "entry");
    self.builder.position_at_end(basic_block);

    let scope_id = self.register_scope(&func.arguments);

    self.scopes[scope_id].pointers = Some(value.get_params());

    self.current_function = Some(value.to_owned());
    func.body.generate(self)?;

    Ok(())
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
        DomainMember::ExternFunction(r#extern) => {
          funcs.push(r#extern.generate(self)?)
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
