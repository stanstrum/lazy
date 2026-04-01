use super::*;

impl<'ctx> LazyType<'ctx> {
  pub(crate) fn fn_type(self,
    param_types: &[inkwell::types::BasicMetadataTypeEnum<'ctx>],
    is_var_args: bool,
  ) -> Result<inkwell::types::FunctionType<'ctx>> {
    Ok(match self {
      LazyType::Void(void_type) => void_type.fn_type(param_types, is_var_args),
      LazyType::Int(int_type) => int_type.fn_type(param_types, is_var_args),
      LazyType::Float(float_type) => float_type.fn_type(param_types, is_var_args),
      LazyType::Pointer(pointer_type) => pointer_type.fn_type(param_types, is_var_args),
    })
  }

  pub(crate) fn into_int_type(self) -> Result<inkwell::types::IntType<'ctx>> {
    match self {
      LazyType::Void(_) => todo!(),
      LazyType::Int(int_type) => Ok(int_type),
      LazyType::Float(_) => todo!(),
      LazyType::Pointer(_) => todo!(),
    }
  }
}

impl<'ctx> LazyValue<'ctx> {
  pub(crate) fn as_basic_value_enum(self) -> Result<inkwell::values::BasicValueEnum<'ctx>> {
    match self {
      LazyValue::Void => todo!(),
      LazyValue::Int(int_value) => Ok(int_value.into()),
    }
  }
}

impl<'ctx> From<inkwell::types::VoidType<'ctx>> for LazyType<'ctx> {
  fn from(value: inkwell::types::VoidType<'ctx>) -> Self {
    Self::Void(value)
  }
}

impl<'ctx> From<inkwell::types::IntType<'ctx>> for LazyType<'ctx> {
  fn from(value: inkwell::types::IntType<'ctx>) -> Self {
    Self::Int(value)
  }
}

impl<'ctx> From<inkwell::types::FloatType<'ctx>> for LazyType<'ctx> {
  fn from(value: inkwell::types::FloatType<'ctx>) -> Self {
    Self::Float(value)
  }
}

impl<'ctx> From<LazyType<'ctx>> for inkwell::types::BasicMetadataTypeEnum<'ctx> {
  fn from(value: LazyType<'ctx>) -> Self {
    match value {
      LazyType::Void(_) => {
        unimplemented!("BasicMetadataTypeEnum cannot represent VoidType")
      },
      LazyType::Int(int_type) => inkwell::types::BasicMetadataTypeEnum::IntType(int_type),
      LazyType::Float(float_type) => inkwell::types::BasicMetadataTypeEnum::FloatType(float_type),
      LazyType::Pointer(pointer_type) => inkwell::types::BasicMetadataTypeEnum::PointerType(pointer_type)
    }
  }
}
