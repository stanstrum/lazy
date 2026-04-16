use inkwell::types::BasicType;

use super::*;

impl<'ctx> LazyType<'ctx> {
  pub(crate) fn fn_type(self,
    param_types: &[inkwell::types::BasicMetadataTypeEnum<'ctx>],
    is_var_args: bool,
  ) -> inkwell::types::FunctionType<'ctx> {
    match self {
      LazyType::Void(void_type) => void_type.fn_type(param_types, is_var_args),
      LazyType::Int(int_type) => int_type.fn_type(param_types, is_var_args),
      LazyType::Float(float_type) => float_type.fn_type(param_types, is_var_args),
      LazyType::Pointer(pointer_type) => pointer_type.fn_type(param_types, is_var_args),
    }
  }

  pub(crate) fn into_int_type(self) -> Option<inkwell::types::IntType<'ctx>> {
    match self {
      LazyType::Void(_) => None,
      LazyType::Int(int_type) => Some(int_type),
      LazyType::Float(_) => None,
      LazyType::Pointer(_) => None,
    }
  }

  pub(crate) fn as_basic_type_enum(self) -> Option<inkwell::types::BasicTypeEnum<'ctx>> {
    match self {
      LazyType::Void(_) => None,
      LazyType::Int(int_type) => Some(int_type.as_basic_type_enum()),
      LazyType::Float(float_type) => Some(float_type.as_basic_type_enum()),
      LazyType::Pointer(pointer_type) => Some(pointer_type.as_basic_type_enum()),
    }
  }
}

impl<'ctx> LazyValue<'ctx> {
  pub(crate) fn as_basic_value_enum(self) -> Option<inkwell::values::BasicValueEnum<'ctx>> {
    match self {
      LazyValue::Void => None,
      LazyValue::Int(int_value) => Some(int_value.into()),
      LazyValue::Pointer(pointer_value) => Some(pointer_value.into()),
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

impl<'ctx> From<inkwell::types::BasicTypeEnum<'ctx>> for LazyType<'ctx> {
  fn from(value: inkwell::types::BasicTypeEnum<'ctx>) -> Self {
    match value {
      inkwell::types::BasicTypeEnum::ArrayType(_) => todo!(),
      inkwell::types::BasicTypeEnum::FloatType(float_type) => Self::Float(float_type),
      inkwell::types::BasicTypeEnum::IntType(int_type) => Self::Int(int_type),
      inkwell::types::BasicTypeEnum::PointerType(pointer_type) => Self::Pointer(pointer_type),
      inkwell::types::BasicTypeEnum::StructType(_) => todo!(),
      inkwell::types::BasicTypeEnum::VectorType(_) => todo!(),
      inkwell::types::BasicTypeEnum::ScalableVectorType(_) => todo!(),
    }
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

impl<'ctx> From<inkwell::values::BasicValueEnum<'ctx>> for LazyValue<'ctx> {
  fn from(value: inkwell::values::BasicValueEnum<'ctx>) -> Self {
    match value {
      inkwell::values::BasicValueEnum::ArrayValue(_) => todo!(),
      inkwell::values::BasicValueEnum::IntValue(int_type) => Self::Int(int_type),
      inkwell::values::BasicValueEnum::FloatValue(_) => todo!(),
      inkwell::values::BasicValueEnum::PointerValue(pointer_value) => Self::Pointer(pointer_value),
      inkwell::values::BasicValueEnum::StructValue(_) => todo!(),
      inkwell::values::BasicValueEnum::VectorValue(_) => todo!(),
      inkwell::values::BasicValueEnum::ScalableVectorValue(_) => todo!(),
    }
  }
}
