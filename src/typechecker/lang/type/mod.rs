pub(crate) mod intrinsics;

use std::cell::RefCell;
use std::rc::Rc;

use crate::asterizer::ast;
use crate::typechecker::{
  preprocess::Preprocess,
  Preprocessor,
  TypeCheckerError,
};

use crate::typechecker::{
  DomainReference,
  lang::Value,
  lang::intrinsics::Intrinsic,
};

#[allow(unused)]
#[derive(Debug, Clone)]
pub(crate) enum Type {
  Intrinsic(Intrinsic),
  Unresolved {
    implied: bool,
    reference: DomainReference,
    template: Option<Vec<TypeCell>>,
  },
  UnsizedArrayOf(TypeCell),
  SizedArrayOf {
    count: Value,
    ty: TypeCell,
  },
  ReferenceTo {
    r#mut: bool,
    ty: TypeCell,
  },
  Shared(TypeCell),
  Function {
    args: Vec<TypeCell>,
    return_ty: TypeCell,
  },
  Struct(Vec<TypeCell>),
  FuzzyInteger,
  FuzzyString {
    size: usize,
    element_ty: Intrinsic,
  },
  Unknown,
}

pub(crate) type TypeCell = Rc<RefCell<Type>>;

impl From<Type> for TypeCell {
  fn from(value: Type) -> Self {
    Rc::new(RefCell::new(value))
  }
}

impl Preprocess for Option<&ast::Type> {
  type Out = Type;

  fn preprocess(&self, preprocessor: &mut Preprocessor) -> Result<Self::Out, TypeCheckerError> {
    match self {
      Some(ast) => ast.preprocess(preprocessor),
      None => Ok(Type::Intrinsic(intrinsics::Intrinsic::Void)),
    }
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
          template
        }) => {
          if !implied && parts.len() == 1 && template.is_none() {
            if let Ok(intrinsic) = intrinsics::Intrinsic::try_from(parts.first().unwrap().as_str()) {
              return Ok(Type::Intrinsic(intrinsic));
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

          Type::Unresolved {
            implied: *implied,
            reference: preprocessor.reference.to_owned(),
            template: template_tys,
          }
        },
        ast::Type::SizedArrayOf(ast::SizedArrayOf { expr, ty }) => {
          let count = Value::Instruction(Box::new(
            expr.preprocess(preprocessor)?
          ));

          let ty = ty.preprocess(preprocessor)?.into();

          Type::SizedArrayOf { count, ty }
        },
        ast::Type::UnsizedArrayOf(ast::UnsizedArrayOf { ty }) => Type::UnsizedArrayOf(
          ty.preprocess(preprocessor)?.into()
        ),
        ast::Type::ImmutableReferenceTo(ast::ImmutableReferenceTo { ty }) => {
          Type::ReferenceTo {
            r#mut: false,
            ty: ty.preprocess(preprocessor)?.into()
          }
        },
      }
    })
  }
}
