use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  Expression,
  Type,
};

use crate::tokenizer::{
  Keyword,
  Operator,
  Punctuation,
  Span,
  GetSpan,
  TokenEnum,
};

use crate::asterizer::error::*;

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct Binding {
  pub(crate) r#mut: bool,
  pub(crate) name: String,
  pub(crate) ty: Option<Type>,
  pub(crate) expr: Option<Expression>,
  pub(crate) span: Span,
}

impl GetSpan for Binding {
  fn get_span(&self) -> &Span {
    todo!()
  }
}

impl MakeAst for Binding {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
        let r#mut = if let Some(TokenEnum::Keyword(Keyword::Mut)) = stream.peek_variant() {
      stream.seek();
      stream.skip_whitespace_and_comments();

      true
    } else {
      false
    };

    let Some(TokenEnum::Identifier(name)) = stream.next_variant() else {
      return Ok(None);
    };

    let name = name.to_owned();

    stream.skip_whitespace_and_comments();

    let ty = {
      if let Some(TokenEnum::Punctuation(Punctuation::Colon)) = stream.peek_variant() {
        stream.seek();
        stream.skip_whitespace_and_comments();

        let Some(ty) = stream.make()? else {
          return ExpectedSnafu {
            what: "a type",
            span: stream.span()
          }.fail();
        };

        Some(ty)
      } else {
        None
      }
    };

    stream.skip_whitespace_and_comments();

    let expr = {
      if let Some(TokenEnum::Operator(Operator::BindAssign)) = stream.peek_variant() {
        stream.seek();
        stream.skip_whitespace_and_comments();

        let Some(expr) = stream.make()? else {
          return ExpectedSnafu {
            what: "an expression",
            span: stream.span()
          }.fail();
        };

        Some(expr)
      } else {
        None
      }
    };

    if let (None, None) = (&ty, &expr) {
      return Ok(None);
    };

    Ok(Some(Self {
      r#mut,
      name,
      ty,
      expr,
      span: stream.span_mark(),
     }))
  }
}
