pub(crate) mod intrinsics;

use std::cell::RefCell;
use std::rc::Rc;

use crate::asterizer::ast;
use crate::tokenizer::{
  GetSpan,
  Span
};

use crate::typechecker::{
  preprocess::{
    Preprocess,
    PreprocessExpression,
  },
  Preprocessor,
  TypeCheckerError,
  DomainReference,
  lang::Value,
  lang::intrinsics::Intrinsic,
};

#[allow(unused)]
#[derive(Debug, Clone)]
pub(crate) enum Type {
  Intrinsic {
    kind: Intrinsic,
    span: Span,
  },
  Unresolved {
    implied: bool,
    reference: DomainReference,
    template: Option<Vec<TypeCell>>,
    span: Span,
  },
  UnsizedArrayOf {
    ty: TypeCell,
    span: Span,
  },
  SizedArrayOf {
    count: Value,
    ty: TypeCell,
    span: Span,
  },
  ReferenceTo {
    r#mut: bool,
    ty: TypeCell,
    span: Span,
  },
  Shared(TypeCell),
  Function {
    args: Vec<TypeCell>,
    return_ty: TypeCell,
    span: Span,
  },
  Struct {
    members: Vec<TypeCell>,
    span: Span,
  },
  FuzzyInteger {
    span: Span,
  },
  FuzzyString {
    size: usize,
    element_ty: Intrinsic,
    span: Span,
  },
  Unknown {
    span: Span,
  },
}

impl GetSpan for Type {
  fn get_span(&self) -> Span {
    match self {
      Type::Shared(_) => todo!(),
      | Type::Intrinsic { span, .. }
      | Type::Unresolved { span, .. }
      | Type::UnsizedArrayOf { span, .. }
      | Type::SizedArrayOf { span, .. }
      | Type::ReferenceTo { span, .. }
      | Type::Function { span, .. }
      | Type::Struct { span, .. }
      | Type::FuzzyInteger { span, .. }
      | Type::FuzzyString { span, .. }
      | Type::Unknown { span } => *span,
    }
  }
}

pub(crate) type TypeCell = Rc<RefCell<Type>>;

impl From<Type> for TypeCell {
  fn from(value: Type) -> Self {
    Rc::new(RefCell::new(value))
  }
}

impl Preprocess for ast::Type {
  type Out = Type;

  fn preprocess(&self, preprocessor: &mut Preprocessor) -> Result<Self::Out, TypeCheckerError> {
    Ok({
      match self {
        ast::Type::Qualified(ast::QualifiedName {
          implied,
          parts,
          template,
          span,
        }) => {
          if !implied && parts.len() == 1 && template.is_none() {
            if let Ok(kind) = intrinsics::Intrinsic::try_from(parts.first().unwrap().as_str()) {
              return Ok(Type::Intrinsic {
                kind,
                span: self.get_span().to_owned(),
              });
            };
          };

          let template_tys = if let Some(template) = template {
            Some({
              template.iter()
                .map(
                  |ty| ty.preprocess(preprocessor).map(Into::into)
                )
                .collect::<Result<_, _>>()?
            })
          } else {
            None
          };

          let mut reference = preprocessor.reference.to_owned();

          if !*implied {
            reference.inner.extend_from_slice(parts.clone().as_slice());
          };

          Type::Unresolved {
            implied: *implied,
            reference,
            template: template_tys,
            span: span.to_owned(),
          }
        },
        ast::Type::SizedArrayOf(ast::SizedArrayOf {
          expr,
          ty,
          span,
        }) => {
          let count = Value::Instruction(Box::new(
            expr.preprocess(preprocessor, &Type::Unknown { span: *span }.into())?
          ));

          let ty = ty.preprocess(preprocessor)?.into();

          Type::SizedArrayOf {
            count,
            ty,
            span: span.to_owned(),
          }
        },
        ast::Type::UnsizedArrayOf(ast::UnsizedArrayOf { ty, span }) => Type::UnsizedArrayOf {
          ty: ty.preprocess(preprocessor)?.into(),
          span: span.to_owned(),
        },
        ast::Type::ImmutableReferenceTo(ast::ImmutableReferenceTo { ty, span }) => {
          Type::ReferenceTo {
            r#mut: false,
            ty: ty.preprocess(preprocessor)?.into(),
            span: span.to_owned(),
          }
        },
      }
    })
  }
}
