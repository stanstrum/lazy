use crate::Compiler;
use crate::token::StringKind;
use crate::span::{GetSpan, Span};
use crate::intrinsic::Intrinsic;
use crate::reference::{AliasReference, BlockReference, ExpressionReference, Reference, Store, StructReference, TypePartReference, TypeReference, VariableReference};

pub trait TypeOf<C: Compiler>: GetSpan<C> {
  fn type_of(&self, store: &C::Store<'_>) -> Option<Type<C>>;
  fn reference(&self, store: &C::Store<'_>) -> Option<OverwriteTypeReference<C>>;
}

#[derive(Debug, Clone, Copy)]
pub enum TypePairModifier {
  Dereference,
}

#[derive(Debug, Clone)]
pub struct OverwriteTypeReference<C: Compiler> {
  pub reference: TypeReference<C>,
  pub modifiers: Vec<TypePairModifier>,
}

#[derive(Debug, Clone)]
pub struct TypePair<C: Compiler> {
  pub overwrite: OverwriteTypeReference<C>,
  pub ty: Type<C>,
}

impl<C: Compiler> TypePair<C> {
  pub fn new(reference: TypeReference<C>, ty: Type<C>) -> Self where OverwriteTypeReference<C>: From<TypeReference<C>> {
    Self {
      overwrite: reference.into(),
      ty,
    }
  }
}

#[derive(Debug, Clone)]
pub enum QualifiedSearchSpace<C: Compiler> {
  Implicit,
  Struct(StructReference<C>),
  Type(OverwriteTypeReference<C>),
  Intrinsic {
    kind: crate::intrinsic::Intrinsic,
    span: crate::span::Span<C>,
  },
  Module(C::ModuleReference),
}

#[derive(Debug, Clone)]
pub struct Qualified<C: Compiler> {
  pub implicit: crate::ty::QualifiedSearchSpace<C>,
  pub parts: Vec<crate::module::Name<C>>,
  pub span: crate::span::Span<C>,
}

impl<C: Compiler> Qualified<C> {
  pub fn is_implicit(&self) -> bool {
    matches!(&self.implicit, QualifiedSearchSpace::Implicit)
  }
}

#[derive(Debug, Clone)]
pub enum Type<C: Compiler> {
  Reference(crate::reference::TypeReference<C>),
  Resolved {
    part: TypePartReference<C>,
    span: Span<C>,
  },
  Unresolved {
    module: C::ModuleReference,
    qualified: Qualified<C>,
  },
  Intrinsic {
    kind: Intrinsic,
    span: Span<C>,
  },
  // Resolved {
  //   original: Box<Type>,
  //   reference: TypeReference,
  // },
  WeakInteger {
    span: Span<C>,
  },
  WeakFloat {
    span: Span<C>,
  },
  WeakString {
    kind: StringKind,
    characters: usize,
    span: Span<C>,
    dereferenced: bool,
  },
  Weak {
    span: Span<C>,
  },
  ReferenceTo {
    ty: TypePartReference<C>,
    r#mut: bool,
    span: Span<C>,
  },
  UnsizedArrayOf {
    ty: TypePartReference<C>,
    span: Span<C>,
  },
  SizedArrayOf {
    ty: TypePartReference<C>,
    size: usize,
    span: Span<C>,
  },
  Struct {
    prototype: StructReference<C>,
  },
}

impl<C: Compiler> TypeReference<C> {
  pub fn parent_module(&self, store: &C::Store<'_>) -> C::ModuleReference {
    match self {
      &TypeReference::Alias(AliasReference(module_reference, _))
        => module_reference,
      &TypeReference::Part(TypePartReference(module_reference, _))
        => module_reference,
      | TypeReference::Expression(ExpressionReference(BlockReference(function_reference, _), _))
      | TypeReference::ReturnTypeOf(function_reference) => {
        let function = function_reference.rget_from(store);
        function.parent
      },
      TypeReference::Variable(v) => store.rget(v.parent()).parent,
      TypeReference::Block(BlockReference(function, _)) => {
        function.rget_from(store).parent
      },
      &TypeReference::StructMember(StructReference(parent, _), _) => parent,
    }
  }
}

impl<C: Compiler> VariableReference<C> {
  pub fn parent(&self) -> C::FunctionReference {
    match self {
      VariableReference::Block(block_reference, _) => block_reference.0,
      VariableReference::Argument(function_reference, _) => *function_reference,
    }
  }
}
