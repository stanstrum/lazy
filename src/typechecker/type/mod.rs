pub(crate) mod intrinsics;

use crate::asterizer::ast;

#[allow(unused)]
#[derive(Debug)]
pub(crate) enum Type {
  Intrinsic(intrinsics::Intrinsic),
  Qualified {
    implied: bool,
    parts: Vec<String>,
    template: Option<Vec<Type>>,
  },
  UnsizedArrayOf(Box<Type>),
}

impl From<Option<&ast::Type>> for Type {
  fn from(value: Option<&ast::Type>) -> Self {
    match value {
      Some(value) => value.into(),
      None => Self::Intrinsic(intrinsics::Intrinsic::Void),
    }
  }
}

impl From<&ast::Type> for Type {
  fn from(value: &ast::Type) -> Self {
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

        Type::Qualified {
          implied: *implied,
          parts: parts.clone(),
          template: template.as_ref()
            .map(
              |v| v.iter()
                .map(Into::into)
                .collect()
            ),
        }
      },
      ast::Type::SizedArrayOf(_) => todo!("from type sizedarrayof"),
      ast::Type::UnsizedArrayOf(ast::UnsizedArrayOf { ty }) => Self::UnsizedArrayOf(
        Box::new(
          ty.as_ref().into()
        )
      ),
      ast::Type::ImmutableReferenceTo(_) => todo!("from type immutablereferenceto"),
    }
  }
}
