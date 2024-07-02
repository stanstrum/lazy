use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError,
  Expression,
};

use crate::tokenizer::{
  Keyword,
  Span,
  GetSpan,
  TokenEnum,
};

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct Return {
  pub(crate) expr: Option<Box<Expression>>,
  pub(crate) span: Span,
}

impl GetSpan for Return {
  fn get_span(&self) -> &Span {
    todo!()
  }
}

impl MakeAst for Return {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
        let Some(TokenEnum::Keyword(Keyword::Return)) = stream.next_variant() else {
      return Ok(None);
    };

    stream.push_mark();
    stream.skip_whitespace_and_comments();

    let expr = stream.make()?.map(Box::new);

    if expr.is_some() {
      stream.drop_mark();
    } else {
      stream.pop_mark();
    };

    Ok(Some(Self {
      expr,
      span: stream.span_mark(),
    }))
  }
}
