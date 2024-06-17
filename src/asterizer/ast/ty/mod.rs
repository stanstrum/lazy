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
  Punctuation,
};

use crate::asterizer::error::*;

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct QualifiedName {
  pub(crate) implied: bool,
  pub(crate) parts: Vec<String>,
  pub(crate) template: Option<Vec<Type>>,
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
  Qualified(QualifiedName),
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

impl MakeAst for QualifiedName {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let implied = {
      if let Some(TokenEnum::Operator(Operator::Separator)) = stream.peek_variant() {
        stream.seek();
        stream.skip_whitespace_and_comments();

        true
      } else {
        false
      }
    };

    let Some(TokenEnum::Identifier(first)) = stream.next_variant() else {
      return if implied {
        ExpectedSnafu {
          what: "an identifier",
          span: stream.span(),
        }.fail()
      } else {
        Ok(None)
      }
    };

    let first = first.to_owned();

    stream.push_mark();
    stream.skip_whitespace_and_comments();

    let mut parts = vec![first];
    loop {
      let Some(TokenEnum::Operator(Operator::Separator)) = stream.next_variant() else {
        stream.pop_mark();
        break;
      };
      stream.drop_mark();
      stream.skip_whitespace_and_comments();

      let Some(TokenEnum::Identifier(part)) = stream.next_variant() else {
        return ExpectedSnafu { 
          what: "an identifier",
          span: stream.span(),
        }.fail();
      };
      let part = part.to_owned();

      parts.push(part);

      stream.push_mark();
      stream.skip_whitespace_and_comments();
    };

    stream.push_mark();
    stream.skip_whitespace_and_comments();

    let template = 'template: {
      if let Some(TokenEnum::Operator(Operator::LessThan)) = stream.next_variant() {
        let mut types = vec![];
        loop {
          stream.skip_whitespace_and_comments();

          let Some(ty) = stream.make()? else {
            stream.pop_mark();
            break 'template None;
          };

          types.push(ty);
          stream.skip_whitespace_and_comments();

          match stream.next_variant() {
            Some(TokenEnum::Punctuation(Punctuation::Comma)) => {},
            Some(TokenEnum::Operator(Operator::GreaterThan)) => {
              stream.drop_mark();
              break;
            },
            _ => {
              stream.pop_mark();
              break 'template None;
            }
          };
        };

        Some(types)
      } else {
        stream.pop_mark();
        None
      }
    };

    return Ok(Some(Self {
      implied,
      parts,
      template,
    }));
  }
}

impl MakeAst for Type {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    #[allow(clippy::manual_map)]
    Ok({
      if let Some(qualified) = stream.make()? {
        Some(Self::Qualified(qualified))
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
