import_export!(function_decl);
import_export!(function_decl_args);

use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  Block,
};

use crate::tokenizer::{
  Span,
  GetSpan,
};

use crate::asterizer::error::*;

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct Function {
  pub(crate) decl: FunctionDeclaration,
  pub(crate) body: Block,
  pub(crate) span: Span,
}

impl GetSpan for Function {
  fn get_span(&self) -> &Span {
    &self.span
  }
}

impl MakeAst for Function {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
        let Some(decl) = stream.make()? else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let Some(body) = stream.make()? else {
      return ExpectedSnafu {
        what: "a block expression",
        span: stream.span()
      }.fail();
    };

    Ok(Some(Self {
      decl,
      body,
      span: stream.span_mark(),
    }))
  }
}
