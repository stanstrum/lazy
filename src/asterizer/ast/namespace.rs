use std::collections::HashMap;

use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError,
  Structure
};

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct Namespace {
  pub name: String,
  children: HashMap<String, Structure>
}

impl MakeAst for Namespace {
  fn make(_stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    eprintln!("{}:{}: Namespace::make empty stub", file!(), line!());

    Ok(None)
  }
}
