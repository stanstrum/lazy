use impls::lang::{Reference, SearchIn};
use inkwell::builder;
use inkwell::context::ContextRef;
use inkwell::types::{BasicMetadataTypeEnum, BasicTypeEnum, FunctionType, MetadataType, VoidType};
use inkwell::values::AsValueRef;

use super::*;

use crate::{compiler, ok};
use crate::translator::lang;

trait TypeOf {
  type Out<'ctx>;

  fn g_type_of<'ctx>(&self, context: &ContextRef<'ctx>) -> Result<Self::Out<'ctx>>;
}

impl<T: TypeOf> TypeOf for RcCell<T> {
  type Out<'ctx> = T::Out<'ctx>;

  fn g_type_of<'ctx>(&self, context: &ContextRef<'ctx>) -> Result<Self::Out<'ctx>> {
    self.borrow().g_type_of(context)
  }
}

trait GeneratorTypeMethods<'ctx> {
  fn fn_type(&self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx>;
  fn into_basic_metadata_type(&self) -> BasicMetadataTypeEnum<'ctx>;
}

impl<'ctx> GeneratorTypeMethods<'ctx> for BasicTypeEnum<'ctx> {
  fn fn_type(&self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx> {
    match self{
      BasicTypeEnum::ArrayType(array_type) => array_type.fn_type(param_types, is_var_args),
      BasicTypeEnum::FloatType(float_type) => float_type.fn_type(param_types, is_var_args),
      BasicTypeEnum::IntType(int_type) => int_type.fn_type(param_types, is_var_args),
      BasicTypeEnum::PointerType(pointer_type) => pointer_type.fn_type(param_types, is_var_args),
      BasicTypeEnum::StructType(struct_type) => struct_type.fn_type(param_types, is_var_args),
      BasicTypeEnum::VectorType(vector_type) => vector_type.fn_type(param_types, is_var_args),
    }
  }

  fn into_basic_metadata_type(&self) -> BasicMetadataTypeEnum<'ctx> {
    todo!()
  }
}

impl<'ctx> GeneratorTypeMethods<'ctx> for GeneratorSuperType<'ctx> {
  fn fn_type(&self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx> {
    match self {
      GeneratorSuperType::Void(void_type) => void_type.fn_type(param_types, is_var_args),
      GeneratorSuperType::Basic(basic_type_enum) => basic_type_enum.fn_type(param_types, is_var_args),
      GeneratorSuperType::Metadata(metadata_type) => metadata_type.fn_type(param_types, is_var_args),
    }
  }

  fn into_basic_metadata_type(&self) -> BasicMetadataTypeEnum<'ctx> {
    todo!()
  }
}

impl<'ctx> GeneratorTypeMethods<'ctx> for BasicMetadataTypeEnum<'ctx> {
  fn fn_type(&self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx> {
    match self {
      BasicMetadataTypeEnum::ArrayType(array_type) => array_type.fn_type(param_types, is_var_args),
      BasicMetadataTypeEnum::FloatType(float_type) => float_type.fn_type(param_types, is_var_args),
      BasicMetadataTypeEnum::IntType(int_type) => int_type.fn_type(param_types, is_var_args),
      BasicMetadataTypeEnum::PointerType(pointer_type) => pointer_type.fn_type(param_types, is_var_args),
      BasicMetadataTypeEnum::StructType(struct_type) => struct_type.fn_type(param_types, is_var_args),
      BasicMetadataTypeEnum::VectorType(vector_type) => vector_type.fn_type(param_types, is_var_args),
      BasicMetadataTypeEnum::MetadataType(metadata_type) => metadata_type.fn_type(param_types, is_var_args),
    }
  }

  fn into_basic_metadata_type(&self) -> BasicMetadataTypeEnum<'ctx> {
    todo!()
  }
}

enum GeneratorSuperType<'ctx> {
  Void(VoidType<'ctx>),
  Basic(BasicTypeEnum<'ctx>),
  Metadata(MetadataType<'ctx>),
}

impl TypeOf for lang::Intrinsic {
  type Out<'ctx> = GeneratorSuperType<'ctx>;

  fn g_type_of<'ctx>(&self, context: &ContextRef<'ctx>) -> Result<Self::Out<'ctx>> {
    Ok(match self {
      lang::Intrinsic::Void => GeneratorSuperType::Void(context.void_type()),
      lang::Intrinsic::U8 => todo!(),
      lang::Intrinsic::U16 => todo!(),
      lang::Intrinsic::U32 => todo!(),
      lang::Intrinsic::U64 => todo!(),
      lang::Intrinsic::I8 => todo!(),
      lang::Intrinsic::I16 => todo!(),
      lang::Intrinsic::I32 => todo!(),
      lang::Intrinsic::I64 => todo!(),
      lang::Intrinsic::F16 => todo!(),
      lang::Intrinsic::F32 => todo!(),
      lang::Intrinsic::F64 => todo!(),
    })
  }
}

impl TypeOf for lang::Type<Module> {
  type Out<'ctx> = GeneratorSuperType<'ctx>;

  fn g_type_of<'ctx>(&self, context: &ContextRef<'ctx>) -> Result<Self::Out<'ctx>> {
    match self {
      lang::Type::Intrinsic { kind, .. } => kind.g_type_of(context),
      lang::Type::Reference(reference) => todo!(),
    }
  }
}

impl TypeOf for lang::Function {
  type Out<'ctx> = FunctionType<'ctx>;

  fn g_type_of<'ctx>(&self, context: &ContextRef<'ctx>) -> Result<Self::Out<'ctx>> {
    let param_types: Vec<_> = self.arguments.iter()
      .map(|argument| {
        Ok(argument.borrow().ty.g_type_of(context)?.into_basic_metadata_type())
      })
      .collect::<Result<_>>()?;

    Ok(self.return_ty.g_type_of(context)?.fn_type(&param_types, false))
  }
}

impl lang::FunctionBlock {
  fn generate_in_function<'ctx, W: CompilerWorkflow>(this: &RcCell<Self>, generator: &mut Generator<W>, function: &FunctionValue<'ctx>) -> Result {
    let builder = generator.context.create_builder();
    let basic_block = generator.context.append_basic_block(*function, "entry");
    builder.position_at_end(basic_block);

    warn!("FunctionBlock stub");

    dbg!(function.get_type().get_return_type());
    builder.build_return(None);

    ok
  }
}

impl lang::Function {
  fn generate_in_module<'ctx, W: CompilerWorkflow>(this: &RcCell<Self>, generator: &mut Generator<W>, module: &inkwell::module::Module<'ctx>) -> Result {
    let context = module.get_context();

    let function = if this.borrow().generator_id.is_some() {
      let id = { this.borrow().generator_id.clone().unwrap() };

      generator.functions[id].clone()
    } else {
      let function_ty = this.g_type_of(&context)?;
      let function = module.add_function(&this.borrow().name.name, function_ty, None);

      trace!("{function:?}");

      let id = generator.functions.len();
      generator.functions.push(unsafe { std::mem::transmute(function.clone()) });

      this.borrow_mut().generator_id = Some(id);

      module.add_function(
        &this.borrow().name.name,
        this.g_type_of(&context)?,
        None
      );

      function
    };

    lang::FunctionBlock::generate_in_function(&this.borrow().body, generator, &function)
  }
}

impl lang::ModuleChild {
  fn generate_in_module<'ctx, W: CompilerWorkflow>(&self, generator: &mut Generator<W>, module: &inkwell::module::Module<'ctx>) -> Result {
    match self {
      ModuleChild::Function(rc) => lang::Function::generate_in_module(rc, generator, module),
      ModuleChild::Module(rc) => rc.borrow().generate_in_module(generator, module),
    }
  }
}

impl lang::Module {
  fn generate_in_module<'ctx, W: CompilerWorkflow>(&self, generator: &mut Generator<W>, module: &inkwell::module::Module<'ctx>) -> Result {
    for child in self.children.iter() {
      child.borrow().generate_in_module(generator, &module)?;
    };

    ok
  }

  pub(in crate::pipeline::generator) fn generate<
    'ctx,
    W: CompilerWorkflow
  >(&self, generator: &mut Generator<W>, context: &'ctx Context) -> Result<inkwell::module::Module<'ctx>> {
    let module = context.create_module(format!("{:?}", &self.name).as_str());

    self.generate_in_module(generator, &module)?;

    Ok(module)
  }
}
