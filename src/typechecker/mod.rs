mod error;

use std::collections::HashMap;
use std::path::Path;

use crate::asterizer::ast::GlobalNamespace;

use crate::compiler::{
  Compiler,
  Handle,
};

pub(crate) use error::*;

#[allow(unused)]
#[derive(Debug, Clone)]
enum Intrinsic {
  U8,
  I8,
  U16,
  I16,
  U32,
  I32,
  U64,
  I64,
  F32,
  F64,
}

#[allow(unused)]
#[derive(Debug, Clone)]
struct StructField {
  name: String,
  ty: Type,
}

#[allow(unused)]
#[derive(Debug, Clone)]
struct NamedVariant {
  name: String,
  ty: Type,
}

#[allow(unused)]
#[derive(Debug, Clone)]
enum Type {
  Intrinsic(Intrinsic),
  ReferenceTo(Box<Type>),
  MutReferenceTo(Box<Type>),
  UnsizedArrayOf(Box<Type>),
  SizedArrayOf {
    size: usize,
    ty: Box<Type>,
  },
  Struct(Vec<StructField>),
  NamedEnum(Vec<NamedVariant>),
  UnnamedEnum(Vec<Type>),
}

#[allow(unused)]
enum TypeScopeChild {
  Type(Type),
  Scope(TypeScope),
}

#[allow(unused)]
struct TypeScope {
  children: HashMap<String, TypeScopeChild>,
}

#[allow(unused)]
pub(crate) struct TypeChecker {
  scope: TypeScope,
}

impl TypeChecker {
  fn new() -> Self {
    Self {
      scope: TypeScope {
        children: HashMap::new(),
      },
    }
  }
}

pub(crate) fn typecheck(compiler: &mut Compiler, path: &Path, handle: &Handle, ast: GlobalNamespace) -> Result<(), TypeCheckerError> {
  let mut checker = TypeChecker::new();

  todo!("typecheck")
}
