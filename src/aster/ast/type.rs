/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use super::*;
use crate::make_get_span;

use crate::aster::intrinsics::Intrinsic;

#[derive(Debug, Clone)]
pub enum Literal {
  UnicodeString(String),
  ByteString(String),
  CString(String),
  Char(char),
  ByteChar(char),
  FloatLiteral(String),
  IntLiteral(String),
}

#[derive(Debug, Clone)]
pub struct LiteralAST {
  pub span: Span,
  pub l: Literal,
}

#[derive(Debug, Clone)]
pub enum GenericConstraint {
  ExtendsTrait(*const TraitAST)
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum Type {
  Intrinsic(Intrinsic),
  Function(*const FunctionAST),
  External(*const ExternDeclAST),
  Struct(*const StructAST),
  ConstReferenceTo(Box<TypeAST>),
  MutReferenceTo(Box<TypeAST>),
  ConstPtrTo(Box<TypeAST>),
  MutPtrTo(Box<TypeAST>),
  ArrayOf(Option<LiteralAST>, Box<TypeAST>),
  Defined(*const TypeAST),
  Generic(IdentAST, Vec<GenericConstraint>),
  Unknown(QualifiedAST),
  UnresolvedLiteral(Literal),
  Unresolved
}

#[derive(Debug, Clone)]
pub struct TypeAST {
  pub span: Span,
  pub e: Type,
}

impl LiteralAST {
  pub fn to_hashable(&self) -> String {
    match &self.l {
      Literal::FloatLiteral(text) => text.to_owned(),
      Literal::IntLiteral(text) => text.to_owned(),
      _ => panic!("to_hashable run on non-numeric")
    }
  }
}

impl Type {
  pub fn to_hashable(&self) -> String {
    match self {
      Type::Intrinsic(s) => {
        s.get_name()
      },
      Type::ConstReferenceTo(ty) => {
        format!("&{}", ty.e.to_hashable())
      },
      Type::MutReferenceTo(ty) => {
        format!("&mut {}", ty.e.to_hashable())
      },
      Type::ConstPtrTo(ty) => {
        format!("*{}", ty.e.to_hashable())
      },
      Type::MutPtrTo(ty) => {
        format!("*mut {}", ty.e.to_hashable())
      },
      Type::ArrayOf(sz, ty) => {
        match sz {
          Some(sz) => {
            format!("[{}]{}", sz.to_hashable(), ty.e.to_hashable())
          },
          None => {
            format!("[]{}", ty.e.to_hashable())
          },
        }
      },
      Type::Defined(ty) => unsafe {
        (**ty).e.to_hashable()
      },
      Type::Unknown(qual) => {
        qual.to_hashable()
      },
      _ => unimplemented!("to_hashable for {:#?}", self)
    }
  }
}

impl TypeAST {
  pub fn to_hashable(&self) -> String {
    self.e.to_hashable()
  }
}

make_get_span![
  TypeAST,
  LiteralAST
];
