use super::*;

#[allow(unused)]
pub(super) enum GeneratorSuperType<'ctx> {
  Void(VoidType<'ctx>),
  Basic(BasicTypeEnum<'ctx>),
  Metadata(MetadataType<'ctx>),
}

#[allow(unused)]
pub(super) enum ContextOrRef<'a, 'ctx> {
  Context(&'a Context),
  Ref(&'a ContextRef<'ctx>),
}

pub(super) trait GeneratorTypeMethods<'ctx> {
  fn fn_type(&self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx>;
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
  fn fn_type(&self, param_types: &[BasicMetadataTypeEnum<'ctx>], is_var_args: bool) -> FunctionType<'ctx> {
    match self {
      GeneratorSuperType::Void(void_type) => void_type.fn_type(param_types, is_var_args),
      GeneratorSuperType::Basic(basic_type_enum) => basic_type_enum.fn_type(param_types, is_var_args),
      GeneratorSuperType::Metadata(metadata_type) => metadata_type.fn_type(param_types, is_var_args),
    }
  }

  fn as_basic_metadata_type(&self) -> BasicMetadataTypeEnum<'ctx> {
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

  fn as_basic_metadata_type(&self) -> BasicMetadataTypeEnum<'ctx> {
    todo!()
  }
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

  fn as_basic_metadata_type(&self) -> BasicMetadataTypeEnum<'ctx> {
    todo!()
  }
}
