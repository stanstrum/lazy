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
pub enum TemplateConstraint {
  Unconstrained(IdentAST),
  Extends(IdentAST, TypeAST)
  // infer :)
}

impl GetSpan for TemplateConstraint {
  fn span(&self) -> Span {
    match self {
      TemplateConstraint::Unconstrained(ident) => ident.span(),
      TemplateConstraint::Extends(ident, ty) => {
        let ident_span = ident.span();

        Span { end: ty.span.end, ..ident_span }
      },
    }
  }
}

#[derive(Debug, Clone)]
pub struct TemplateAST {
  pub span: Span,
  pub constraints: Vec<TemplateConstraint>
}

#[derive(Debug, Clone)]
pub struct NamespaceAST {
  pub span: Span,
  pub ident: IdentAST,
  // todo: turn this to Hashmap<IdentAST, Structure> for consistency
  pub map: HashMap<String, Structure>,
  pub imports: Vec<ImportAST>,
}

#[derive(Debug, Clone)]
pub struct FunctionAST {
  pub span: Span,
  pub decl: FunctionDeclAST,
  pub body: BlockExpressionAST,

  pub template: Option<TemplateAST>
}

#[derive(Debug, Clone)]
pub struct StructAST {
  pub span: Span,
  pub ident: IdentAST,
  pub members: Vec<(TypeAST, IdentAST)>,
  pub template: Option<TemplateAST>
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
  pub body: BlockExpressionAST,

  pub template: Option<TemplateAST>
}

#[derive(Debug, Clone)]
pub struct TraitAST {
  pub span: Span,
  pub ident: IdentAST,
  pub decls: Vec<MemberFunctionDeclAST>,
  pub template: Option<TemplateAST>
}

#[derive(Debug, Clone)]
pub struct ImplAST {
  pub span: Span,

  // impl ...
  pub ty: TypeAST,
  // {
  pub methods: Vec<MemberFunctionAST>,
  // }

  pub template: Option<TemplateAST>
}

#[derive(Debug, Clone)]
pub struct ImplForAST {
  pub span: Span,

  // impl ...
  pub r#trait: QualifiedAST,
  // for ...
  pub ty: TypeAST,
  // {
  pub methods: Vec<MemberFunctionAST>,
  // }

  pub template: Option<TemplateAST>
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
  pub ty: TypeAST,
  pub template: Option<TemplateAST>
}

#[derive(Debug, Clone)]
pub struct ExternDeclAST {
  pub span: Span,

  pub ident: IdentAST,
  pub ret: TypeAST,
  pub args: HashMap<IdentAST, TypeAST>,

  pub varargs: bool
}

#[derive(Debug, Clone)]
pub enum ImportPatternAST {
  Qualify {
    span: Span,
    ident: IdentAST,
    child: Box<ImportPatternAST>
  },
  Brace {
    span: Span,
    children: Vec<ImportPatternAST>
  },
  Ident {
    span: Span,
    ident: IdentAST, alias: Option<IdentAST>
  }
}

#[derive(Debug, Clone)]
pub struct ImportAST {
  pub span: Span,
  pub pattern: ImportPatternAST,
  pub from: LiteralAST,
  pub ns: NamespaceAST,
}

#[derive(Debug, Clone)]
pub enum Structure {
  Namespace(NamespaceAST),
  Function(FunctionAST),
  Struct(StructAST),
  Trait(TraitAST),
  Impl(Impl),
  TypeAlias(TypeAliasAST),
  ExternDecl(ExternDeclAST),
  ImportedNamespace { ident: IdentAST, span: Span, ns: *mut NamespaceAST },
  ImportedStructure { ident: IdentAST, span: Span, structure: *mut Self },
}

impl GetSpan for Impl {
  fn span(&self) -> Span {
    match self {
      Impl::Impl(s) => s.span.clone(),
      Impl::ImplFor(s) => s.span.clone(),
    }
  }
}

impl GetSpan for ImportPatternAST {
  fn span(&self) -> Span {
    match self {
      ImportPatternAST::Qualify { span, .. } => span.clone(),
      ImportPatternAST::Brace { span, .. } => span.clone(),
      ImportPatternAST::Ident { span, .. } => span.clone(),
    }
  }
}

impl GetSpan for Structure {
  fn span(&self) -> Span {
    match self {
      Structure::Namespace(s) => s.span(),
      Structure::Function(s) => s.span(),
      Structure::Trait(s) => s.span(),
      Structure::Impl(s) => s.span(),
      Structure::TypeAlias(s) => s.span(),
      Structure::Struct(s) => s.span(),
      Structure::ExternDecl(r#extern) => r#extern.span(),
      Structure::ImportedNamespace { span, .. } => span.clone(),
      Structure::ImportedStructure { span, .. } => span.clone(),
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
  ExternDeclAST,
  ImportAST
];
