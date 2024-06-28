pub(crate) mod intrinsics;

use std::rc::Rc;

use crate::asterizer::ast;

use super::super::DomainReference;

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
  ReferenceTo {
    r#mut: bool,
    ty: Box<Type>,
  },
  Shared(Rc<Type>),
  Unknown,
}

impl Type {
  pub(crate) fn from_ast_optional(value: Option<&ast::Type>, reference: &DomainReference) -> Self {
    match value {
      Some(value) => Type::from_ast(value, reference),
      None => Type::Intrinsic(intrinsics::Intrinsic::Void),
    }
  }

  pub(crate) fn from_ast(value: &ast::Type, reference: &DomainReference) -> Self {
    match value {
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
          reference: reference.to_owned(),
          template: template.as_ref()
            .map(
              |tys| tys.iter()
                .map(|ty| Type::from_ast(ty, reference))
                .collect()
            ),
        }
      },
      ast::Type::SizedArrayOf(_) => todo!("from type sizedarrayof"),
      ast::Type::UnsizedArrayOf(ast::UnsizedArrayOf { ty }) => Self::UnsizedArrayOf(
        Box::new(
          Type::from_ast(ty.as_ref(), reference)
        )
      ),
      ast::Type::ImmutableReferenceTo(ast::ImmutableReferenceTo { ty }) => {
        Type::ReferenceTo {
          r#mut: false,
          ty: Box::new(
            Type::from_ast(ty, reference)
          )
        }
      },
    }
  }
}
