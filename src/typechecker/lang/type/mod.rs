pub(crate) mod intrinsics;

use std::rc::Rc;

use crate::{asterizer::ast, typechecker::{preprocess::Preprocess, TypeChecker}};

use super::{super::DomainReference, Value};

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

  fn preprocess(&self, checker: &mut TypeChecker) -> Self::Out {
    match self {
      Some(ast) => ast.preprocess(checker),
      None => Type::Intrinsic(intrinsics::Intrinsic::Void),
    }
  }
}

impl Preprocess for ast::Type {
  type Out = Type;

  fn preprocess(&self, checker: &mut TypeChecker) -> Self::Out {
    match self {
      ast::Type::Qualified(ast::QualifiedName {
        implied,
        parts,
        template
      }) => {
        if !implied && parts.len() == 1 && template.is_none() {
          if let Ok(intrinsic) = intrinsics::Intrinsic::try_from(parts.first().unwrap().as_str()) {
            return Type::Intrinsic(intrinsic);
          };
        };

        Type::Unresolved {
          implied: *implied,
          reference: checker.reference.to_owned(),
          template: template.as_ref()
            .map(
              |tys| tys.iter()
                .map(|ty| ty.preprocess(checker))
                .collect()
            ),
        }
      },
      ast::Type::SizedArrayOf(ast::SizedArrayOf { expr, ty }) => {
        let count = Value::Instruction(Box::new(
          expr.preprocess(checker)
        ));

        let ty = Box::new(ty.preprocess(checker));

        Type::SizedArrayOf { count, ty }
      },
      ast::Type::UnsizedArrayOf(ast::UnsizedArrayOf { ty }) => Type::UnsizedArrayOf(
        Box::new(ty.preprocess(checker))
      ),
      ast::Type::ImmutableReferenceTo(ast::ImmutableReferenceTo { ty }) => {
        Type::ReferenceTo {
          r#mut: false,
          ty: Box::new(
            ty.preprocess(checker)
          )
        }
      },
    }
  }
}
