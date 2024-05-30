use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  Expression,
};

use crate::tokenizer::{
  TokenEnum,
  Grouping,
  GroupingType,
  Operator,
};

use crate::asterizer::error::*;

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct NamedType {
  pub(crate) name: String
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct SizedArrayOf {
  pub(crate) expr: Expression,
  pub(crate) ty: Box<Type>
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct UnsizedArrayOf {
  pub(crate) ty: Box<Type>
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct ImmutableReferenceTo {
  pub(crate) ty: Box<Type>
}

#[derive(Debug, TypeName)]
pub(crate) enum Type {
  Named(NamedType),
  SizedArrayOf(SizedArrayOf),
  UnsizedArrayOf(UnsizedArrayOf),
  ImmutableReferenceTo(ImmutableReferenceTo)
}

impl MakeAst for SizedArrayOf {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(TokenEnum::Grouping(Grouping::Open(GroupingType::Bracket))) = stream.next_variant() else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let Some(expr) = stream.make()? else {
      return Ok(None);
    };

    let Some(TokenEnum::Grouping(Grouping::Close(GroupingType::Bracket))) = stream.next_variant() else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let Some(ty) = stream.make()? else {
      return ExpectedSnafu {
        what: "a type",
        span: stream.span()
      }.fail();
    };

    Ok(Some(Self {
      ty: Box::new(ty),
      expr
    }))
  }
}

impl MakeAst for UnsizedArrayOf {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(TokenEnum::Grouping(Grouping::Open(GroupingType::Bracket))) = stream.next_variant() else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let Some(TokenEnum::Grouping(Grouping::Close(GroupingType::Bracket))) = stream.next_variant() else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let Some(ty) = stream.make()? else {
      return ExpectedSnafu {
        what: "a type",
        span: stream.span()
      }.fail();
    };

    Ok(Some(Self {
      ty: Box::new(ty)
    }))
  }
}

impl MakeAst for ImmutableReferenceTo {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(TokenEnum::Operator(Operator::SingleAnd)) = stream.next_variant() else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let Some(ty) = stream.make()? else {
      return ExpectedSnafu {
        what: "a type",
        span: stream.span()
      }.fail();
    };

    Ok(Some(Self {
      ty: Box::new(ty)
    }))
  }
}

impl MakeAst for NamedType {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(TokenEnum::Identifier(name)) = stream.next_variant() else {
      return Ok(None);
    };

    let name = name.to_owned();

    Ok(Some(Self { name }))
  }
}

impl MakeAst for Type {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    #[allow(clippy::manual_map)]
    Ok({
      if let Some(named) = stream.make()? {
        Some(Self::Named(named))
      } else if let Some(sized_array_of) = stream.make()? {
        Some(Self::SizedArrayOf(sized_array_of))
      } else if let Some(unsized_array_of) = stream.make()? {
        Some(Self::UnsizedArrayOf(unsized_array_of))
      } else if let Some(immut_ref_to) = stream.make()? {
        Some(Self::ImmutableReferenceTo(immut_ref_to))
      } else {
        None
      }
    })
  }
}
