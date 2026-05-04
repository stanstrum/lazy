use crate::token::StringKind;
use crate::span::{GetSpan, Span};
use crate::intrinsic::Intrinsic;
use crate::reference::{AliasReference, BlockReference, ExpressionReference, Reference, Store, StructReference, TypePartReference, TypeReference, VariableReference};
use crate::module::Name;
use crate::Compiler;

pub trait TypeOf<C: Compiler>: GetSpan<C> {
  fn type_of(&self, store: &C::Store<'_>) -> Option<Type<C>>;
}

#[derive(Debug)]
pub struct ResolvedType<C: Compiler> {
  pub reference: TypeReference<C>,
  pub ty: TypeValue<C>,
}

#[derive(Debug, Clone)]
pub struct Type<C: Compiler> {
  pub reference: TypeReference<C>,
  pub ty: Option<TypeValue<C>>,
}

impl<C: Compiler> Type<C> {
  pub fn new(reference: TypeReference<C>, ty: TypeValue<C>) -> Self {
    Self {
      reference: reference.into(),
      ty: Some(ty),
    }
  }
}

impl<C: Compiler> From<TypeReference<C>> for Type<C> {
  fn from(reference: TypeReference<C>) -> Self {
    Self {
      reference,
      ty: None,
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum QualifiedSearchSpace<C: Compiler> {
  Implicit,
  Struct(StructReference<C>),
  Type(TypeReference<C>),
  Intrinsic {
    kind: Intrinsic,
    span: Span<C>,
  },
  Module(C::ModuleReference),
}

#[derive(Debug, Clone)]
pub struct Qualified<C: Compiler> {
  pub implicit: QualifiedSearchSpace<C>,
  pub parts: Vec<Name<C>>,
  pub span: Span<C>,
}

impl<C: Compiler> Qualified<C> {
  pub fn is_implicit(&self) -> bool {
    matches!(&self.implicit, QualifiedSearchSpace::Implicit)
  }
}

#[derive(Debug, Clone)]
pub enum TypeValue<C: Compiler> {
  Reference(TypeReference<C>),
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
