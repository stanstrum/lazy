/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use super::*;
use crate::make_get_span;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct NamespaceAST {
  pub span: Span,
  pub ident: IdentAST,
  pub map: HashMap<String, Structure>,
}

#[derive(Debug, Clone)]
pub struct FunctionAST {
  pub span: Span,
  pub decl: FunctionDeclAST,
  pub body: BlockExpressionAST,
}

#[derive(Debug, Clone)]
pub struct StructAST {
  pub span: Span,
  pub ident: IdentAST,
  pub members: Vec<(TypeAST, IdentAST)>
}

#[derive(Debug, Clone)]
pub struct FunctionDeclAST {
  pub span: Span,
  pub ident: IdentAST,
  pub args: HashMap<IdentAST, TypeAST>,
  pub ret: TypeAST,
}

#[derive(Debug, Clone)]
pub struct MemberFunctionDeclAST {
  pub span: Span,

  pub public: Option<KeywordAST>,
  pub r#static: Option<KeywordAST>,
  pub r#mut: Option<KeywordAST>,

  pub decl: FunctionDeclAST
}

#[derive(Debug, Clone)]
pub struct MemberFunctionAST {
  pub span: Span,

  pub decl: MemberFunctionDeclAST,
  pub body: BlockExpressionAST
}

#[derive(Debug, Clone)]
pub struct TraitAST {
  pub span: Span,
  pub ident: IdentAST,
  pub decls: Vec<MemberFunctionDeclAST>
}

#[derive(Debug, Clone)]
pub struct ImplAST {
  pub span: Span,

  // impl ...
  pub ty: TypeAST,
  // {
  pub methods: Vec<MemberFunctionAST>,
  // }
}

#[derive(Debug, Clone)]
pub struct ImplForAST {
  pub span: Span,

  // impl ...
  pub r#trait: QualifiedAST,
  // for ...
  pub ty: TypeAST,
  // {
  pub methods: Vec<MemberFunctionAST>
  // }
}

#[derive(Debug, Clone)]
pub enum Impl {
  Impl(ImplAST),
  ImplFor(ImplForAST)
}

#[derive(Debug, Clone)]
pub struct TypeAliasAST {
  pub span: Span,
  pub ident: IdentAST,
  pub ty: TypeAST
}

#[derive(Debug, Clone)]
pub struct ExternDeclAST {
  pub span: Span,
  pub decl: FunctionDeclAST
}

#[derive(Debug, Clone)]
pub enum Structure {
  Namespace(NamespaceAST),
  Function(FunctionAST),
  Struct(StructAST),
  Trait(TraitAST),
  Impl(Impl),
  TypeAlias(TypeAliasAST),
  ExternDecl(ExternDeclAST)
}

impl GetSpan for Impl {
  fn span(&self) -> Span {
    match self {
      Impl::Impl(s) => s.span,
      Impl::ImplFor(s) => s.span,
    }
  }
}

impl GetSpan for &Structure {
  fn span(&self) -> Span {
    match self {
      Structure::Namespace(s) => s.span(),
      Structure::Function(s) => s.span(),
      Structure::Trait(s) => s.span(),
      Structure::Impl(s) => s.span(),
      Structure::TypeAlias(s) => s.span(),
      Structure::Struct(s) => s.span(),
      Structure::ExternDecl(r#extern) => r#extern.span(),
    }
  }
}

make_get_span![
  NamespaceAST,
  FunctionDeclAST,
  FunctionAST,
  StructAST,
  MemberFunctionAST,
  TraitAST,
  ImplAST,
  ImplForAST,
  TypeAliasAST,
  ExternDeclAST
];
