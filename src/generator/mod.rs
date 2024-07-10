mod error;

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

use crate::tokenizer;
use crate::typechecker::{
  lang,
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

impl<'a> Generate<'a> for lang::intrinsics::Intrinsic {
  type Out = GeneratorType<'a>;

  fn generate(&self, generator: &mut Generator<'a>) -> Result<Self::Out, GeneratorError> {
    Ok(match self {
      lang::intrinsics::Intrinsic::Void => GeneratorType::Void(generator.context.void_type()),
      | lang::intrinsics::Intrinsic::U8
      | lang::intrinsics::Intrinsic::I8 => GeneratorType::Basic(BasicTypeEnum::IntType(generator.context.i8_type())),
      | lang::intrinsics::Intrinsic::U16
      | lang::intrinsics::Intrinsic::I16 => GeneratorType::Basic(BasicTypeEnum::IntType(generator.context.i16_type())),
      | lang::intrinsics::Intrinsic::U32
      | lang::intrinsics::Intrinsic::I32 => GeneratorType::Basic(BasicTypeEnum::IntType(generator.context.i32_type())),
      | lang::intrinsics::Intrinsic::U64
      | lang::intrinsics::Intrinsic::I64 => GeneratorType::Basic(BasicTypeEnum::IntType(generator.context.i64_type())),
      lang::intrinsics::Intrinsic::F32 => todo!(),
      lang::intrinsics::Intrinsic::F64 => todo!(),
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

impl ResolveToU32 for lang::Value {
  fn resolve_to_u32(&self) -> u32 {
    match self {
      lang::Value::Variable(_) => todo!(),
      lang::Value::Instruction(_) => todo!(),
      lang::Value::Literal(literal) => literal.resolve_to_u32(),
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
      _ => todo!("{self:?}")
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

impl<'a> Generate<'a> for lang::Value {
  type Out = Option<BasicValueEnum<'a>>;

  fn generate(&self, generator: &mut Generator<'a>) -> Result<Self::Out, GeneratorError> {
    Ok(match self {
      lang::Value::Variable(variable) => Some(variable.generate(generator)?),
      lang::Value::Instruction(instruction) => instruction.generate(generator)?,
      lang::Value::Literal(literal) => {
        match &literal.kind {
          tokenizer::LiteralKind::Integer(integer) => Some(
            literal.type_of()
              .unwrap()
              .generate(generator)?
              .into_basic()
              .into_int_type()
              .const_int(*integer, false)
              .as_basic_value_enum()
          ),
          tokenizer::LiteralKind::FloatingPoint(float) => Some(
            literal.type_of()
              .unwrap()
              .generate(generator)?
              .into_basic()
              .into_float_type()
              .const_float(*float)
              .as_basic_value_enum()
          ),
          tokenizer::LiteralKind::UnicodeString(string) => match self.type_of().unwrap() {
            lang::Type::SizedArrayOf { .. } => {
              let char_ty = generator.context.i32_type();

              let values = string.chars()
                .map(|ch| {
                  char_ty.const_int(ch as u64, false)
                })
                .collect::<Vec<_>>();

              Some(char_ty.const_array(&values).as_basic_value_enum())
            },
            | lang::Type::FuzzyString { .. }
            | lang::Type::ReferenceTo { .. } => {
              let char_ty = generator.context.i32_type();

              let values = string.chars()
                .map(|ch| {
                  char_ty.const_int(ch as u64, false)
                })
                .collect::<Vec<_>>();

              let value = char_ty.const_array(&values).as_basic_value_enum();

              let global = generator.module.add_global(
                value.get_type().as_basic_type_enum(),
                Default::default(),
                "unicode_string"
              );

              global.set_initializer(&value);

              Some(global.as_basic_value_enum())
            },
            a => unreachable!("{a:?}"),
          },
          tokenizer::LiteralKind::CString(_) => todo!(),
          tokenizer::LiteralKind::ByteString(_) => todo!(),
          tokenizer::LiteralKind::UnicodeChar(_) => todo!(),
          tokenizer::LiteralKind::ByteChar(_) => todo!(),
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
      lang::Value::Literal(_) => todo!(),
    }
  }

  fn generate_block(&mut self, block: &mut lang::Block, function: FunctionValue<'a>) -> Result<(), GeneratorError> {
    let basic = self.context.append_basic_block(function, "entry");
    self.builder.position_at_end(basic);

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
