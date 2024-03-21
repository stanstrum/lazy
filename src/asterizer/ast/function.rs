use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  BlockExpression,
  FunctionDeclaration
};

use crate::asterizer::error::*;

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct Function {
  pub decl: FunctionDeclaration,
  pub body: BlockExpression
}

impl MakeAst for Function {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(decl) = stream.make::<FunctionDeclaration>()? else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let Some(body) = stream.make::<BlockExpression>()? else {
      return ExpectedSnafu {
        what: "a block expression"
      }.fail();
    };

    Ok(Some(Self { decl, body }))
  }
}
