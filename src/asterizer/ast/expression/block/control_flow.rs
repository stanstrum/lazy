use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError,
  Expression,
  Block,
};

#[derive(Debug, TypeName)]
pub(crate) struct IfBranch {
  clause: Expression,
  body: Block
}

#[derive(Debug, TypeName)]
pub(crate) struct If {
  branches: Vec<IfBranch>,
  r#else: Block
}

#[derive(Debug, TypeName)]
pub(crate) struct While {
  clause: Expression,
  body: Block
}

#[derive(Debug, TypeName)]
pub(crate) struct DoWhile {
  body: Block,
  clause: Expression,
}

#[derive(Debug, TypeName)]
pub(crate) struct For {
  clause: Expression,
  body: Block
}

#[derive(Debug, TypeName)]
pub(crate) struct Until {
  clause: Expression,
  body: Block
}

#[derive(Debug, TypeName)]
pub(crate) struct Loop {
  body: Block
}

#[derive(Debug, TypeName)]
pub(crate) enum ControlFlow {
  If(If),
  While(While),
  DoWhile(DoWhile),
  For(For),
  Until(Until),
  Loop(Loop),
}

impl MakeAst for If {
  fn make(_stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    Ok(None)
  }
}

impl MakeAst for While {
  fn make(_stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    Ok(None)
  }
}

impl MakeAst for DoWhile {
  fn make(_stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    Ok(None)
  }
}

impl MakeAst for For {
  fn make(_stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    Ok(None)
  }
}

impl MakeAst for Until {
  fn make(_stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    Ok(None)
  }
}

impl MakeAst for Loop {
  fn make(_stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    Ok(None)
  }
}

impl MakeAst for ControlFlow {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    #[allow(clippy::manual_map)]
    Ok({
      if let Some(r#if) = stream.make()? {
        Some(r#if)
      } else if let Some(r#while) = stream.make()? {
        Some(r#while)
      } else if let Some(do_while) = stream.make()? {
        Some(do_while)
      } else if let Some(r#for) = stream.make()? {
        Some(r#for)
      } else if let Some(until) = stream.make()? {
        Some(until)
      } else if let Some(r#loop) = stream.make()? {
        Some(r#loop)
      } else {
        None
      }
    })
  }  
}