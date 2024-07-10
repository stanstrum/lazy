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
    BasicMetadataTypeEnum,
    BasicType,
    BasicTypeEnum,
    FunctionType,
    VoidType,
  },
  values::{BasicValueEnum, FunctionValue, PointerValue}
};

use crate::typechecker::{
  lang,
  Domain,
  DomainMember,
  Program,
  TypeOf,
};

pub(crate) use error::*;

#[allow(unused)]
struct GeneratorScope {
  scope: Rc<RefCell<Vec<lang::Variable>>>,
}

#[allow(unused)]
pub(crate) struct Generator<'a> {
  context: &'a Context,
  module: Module<'a>,
  builder: &'a Builder<'a>,
  scopes: Vec<GeneratorScope>,
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
      GeneratorType::Function(_) => panic!("bad cast"),
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
      Self::Intrinsic { kind, .. } => {
        Ok(
          match kind {
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
          }
        )
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

  fn register_scope(&mut self, scope: &mut lang::VariableScope) {
    if scope.generator_id.is_none() {
      let id = self.scopes.len();

      self.scopes.push(GeneratorScope {
        scope: scope.inner.to_owned(),
      });

      scope.generator_id = Some(id);
    };
  }

  fn resolve_dest(&mut self, _value: &lang::Value) -> Result<PointerValue<'a>, GeneratorError> {
    todo!()
  }

  fn resolve_value(&mut self, _value: &lang::Value) -> Result<BasicValueEnum<'a>, GeneratorError> {
    todo!()
  }

  fn generate_block(&mut self, block: &mut lang::Block, function: FunctionValue<'a>) -> Result<(), GeneratorError> {
    let basic = self.context.append_basic_block(function, "entry");
    self.builder.position_at_end(basic);

    self.register_scope(&mut block.variables);

    for inst in block.body.iter() {
      match inst {
        lang::Instruction::Assign { dest, value, .. } => {
          let dest = self.resolve_dest(dest)?;
          let value = self.resolve_value(value)?;

          self.builder.build_store(dest, value);
        },
        lang::Instruction::Call { .. } => todo!(),
        lang::Instruction::Return { .. } => todo!(),
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
