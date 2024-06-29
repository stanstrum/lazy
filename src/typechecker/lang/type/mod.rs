pub(crate) mod intrinsics;

use std::rc::Rc;

use crate::asterizer::ast;
use crate::typechecker::{
  preprocess::Preprocess,
  Preprocessor,
  TypeCheckerError,
};

use super::{
  super::DomainReference,
  Value,
};

#[allow(unused)]
#[derive(Debug)]
pub(crate) enum Type {
  Intrinsic(intrinsics::Intrinsic),
  Unresolved {
    implied: bool,
    reference: DomainReference,
    template: Option<Vec<Type>>,
  },
  UnsizedArrayOf(Box<Type>),
  SizedArrayOf {
    count: Value,
    ty: Box<Type>,
  },
  ReferenceTo {
    r#mut: bool,
    ty: Box<Type>,
  },
  Shared(Rc<Type>),
  Unknown,
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
                  |ty| ty.preprocess(preprocessor)
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

          let ty = Box::new(ty.preprocess(preprocessor)?);

          Type::SizedArrayOf { count, ty }
        },
        ast::Type::UnsizedArrayOf(ast::UnsizedArrayOf { ty }) => Type::UnsizedArrayOf(
          Box::new(ty.preprocess(preprocessor)?)
        ),
        ast::Type::ImmutableReferenceTo(ast::ImmutableReferenceTo { ty }) => {
          Type::ReferenceTo {
            r#mut: false,
            ty: Box::new(
              ty.preprocess(preprocessor)?
            )
          }
        },
      }
    })
  }
}
