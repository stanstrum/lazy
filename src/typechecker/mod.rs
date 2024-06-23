mod error;

use typename::TypeName;

use std::collections::HashMap;
use std::path::Path;

use crate::asterizer::ast;

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
enum Unparsed {
  Namespace(ast::Namespace),
  Function(ast::Function),
  Type(ast::Type),
  Struct(ast::Struct),
  Class(ast::Class),
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
  Unparsed(Unparsed),
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
  current_scope: Vec<String>,
}

impl TryFrom<&str> for Intrinsic {
  type Error = ();

  fn try_from(value: &str) -> Result<Self, ()> {
    match value {
      "i8" => Ok(Self::I8),
      "u8" => Ok(Self::U8),
      "i16" => Ok(Self::I16),
      "u16" => Ok(Self::U16),
      "i32" => Ok(Self::I32),
      "u32" => Ok(Self::U32),
      "i64" => Ok(Self::I64),
      "u64" => Ok(Self::U64),
      "f32" => Ok(Self::F32),
      "f64" => Ok(Self::F64),
      _ => Err(()),
    }
  }
}

enum ModuleChild {
  Function(Function),
  Module(Module),
}

struct Module {
  children: Vec<ModuleChild>,
}

pub(crate) fn typecheck(compiler: &mut Compiler, path: &Path, handle: &Handle, global: ast::GlobalNamespace) -> Result<(), TypeCheckerError> {
  let mut checker = TypeChecker::new();

  checker.register(global)?;

  todo!("typecheck")
}
