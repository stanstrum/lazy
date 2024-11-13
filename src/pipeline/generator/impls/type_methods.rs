use inkwell::types::BasicType;

use super::*;

#[allow(unused)]
pub(super) enum GeneratorSuperType<'ctx> {
  Void(VoidType<'ctx>),
  Basic(BasicTypeEnum<'ctx>),
  Metadata(MetadataType<'ctx>),
}

#[allow(unused)]
#[derive(Clone, Copy)]
pub(super) enum ContextOrRef<'a, 'ctx> {
  Context(&'a Context),
  Ref(&'a ContextRef<'ctx>),
}

pub(super) trait GeneratorTypeMethods<'ctx> {
  fn fn_type(
    &self,
    param_types: &[BasicMetadataTypeEnum<'ctx>],
    is_var_args: bool,
  ) -> FunctionType<'ctx>;
  fn as_basic_type_enum(&self) -> BasicTypeEnum<'ctx>;
  fn as_basic_metadata_type(&self) -> BasicMetadataTypeEnum<'ctx>;
}

impl<'a, 'ctx> From<&'a Context> for ContextOrRef<'a, 'ctx> {
  fn from(context: &'a Context) -> Self {
    Self::Context(context)
  }
}

impl<'a, 'ctx> From<&'a ContextRef<'ctx>> for ContextOrRef<'a, 'ctx> {
  fn from(r#ref: &'a ContextRef<'ctx>) -> Self {
    Self::Ref(r#ref)
  }
}

impl<'a, 'ctx> ContextOrRef<'a, 'ctx> {
  pub(super) fn void_type(&'ctx self) -> VoidType<'ctx> {
    match self {
      ContextOrRef::Context(context) => context.void_type(),
      ContextOrRef::Ref(context) => context.void_type(),
    }
  }
}

impl<'ctx> GeneratorTypeMethods<'ctx> for GeneratorSuperType<'ctx> {
  fn fn_type(
    &self,
    param_types: &[BasicMetadataTypeEnum<'ctx>],
    is_var_args: bool,
  ) -> FunctionType<'ctx> {
    match self {
      GeneratorSuperType::Void(void_type) => void_type.fn_type(param_types, is_var_args),
      GeneratorSuperType::Basic(basic_type_enum) => {
        basic_type_enum.fn_type(param_types, is_var_args)
      },
      GeneratorSuperType::Metadata(metadata_type) => {
        metadata_type.fn_type(param_types, is_var_args)
      },
    }
  }

  fn as_basic_metadata_type(&self) -> BasicMetadataTypeEnum<'ctx> {
    match self {
      GeneratorSuperType::Void(_) => unimplemented!(),
      GeneratorSuperType::Basic(basic_type_enum) => (*basic_type_enum).try_into().unwrap(),
      GeneratorSuperType::Metadata(metadata_type) => {
        BasicMetadataTypeEnum::MetadataType(*metadata_type)
      },
    }
  }

  fn as_basic_type_enum(&self) -> BasicTypeEnum<'ctx> {
    match self {
      | GeneratorSuperType::Void(_) | GeneratorSuperType::Metadata(_) => unimplemented!(),
      GeneratorSuperType::Basic(BasicTypeEnum::ArrayType(array)) => {
        BasicTypeEnum::ArrayType(*array)
      },
      GeneratorSuperType::Basic(BasicTypeEnum::FloatType(float)) => {
        BasicTypeEnum::FloatType(*float)
      },
      GeneratorSuperType::Basic(BasicTypeEnum::IntType(int)) => BasicTypeEnum::IntType(*int),
      GeneratorSuperType::Basic(BasicTypeEnum::PointerType(pointer)) => {
        BasicTypeEnum::PointerType(*pointer)
      },
      GeneratorSuperType::Basic(BasicTypeEnum::StructType(r#struct)) => {
        BasicTypeEnum::StructType(*r#struct)
      },
      GeneratorSuperType::Basic(BasicTypeEnum::VectorType(vector)) => {
        BasicTypeEnum::VectorType(*vector)
      },
    }
  }
}
