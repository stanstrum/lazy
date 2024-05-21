use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError,
  Expression,
  Block,
};

use crate::tokenizer::{
  TokenEnum,
  Keyword,
};

use crate::asterizer::error::ExpectedSnafu;

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
pub(crate) struct Until {
  clause: Expression,
  body: Block
}

#[derive(Debug, TypeName)]
pub(crate) struct DoUntil {
  body: Block,
  clause: Expression,
}

#[derive(Debug, TypeName)]
pub(crate) struct For {
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
  Until(Until),
  DoUntil(DoUntil),
  For(For),
  Loop(Loop),
}

impl MakeAst for If {
  fn make(_stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    Ok(None)
  }
}

impl MakeAst for While {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(TokenEnum::Keyword(Keyword::While)) = stream.next_variant() else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let Some(clause) = stream.make()? else {
      return ExpectedSnafu {
        what: "an expression",
        span: stream.span()
      }.fail();
    };

    stream.skip_whitespace_and_comments();

    let Some(body) = stream.make()? else {
      return ExpectedSnafu {
        what: "a block expression",
        span: stream.span()
      }.fail();
    };
    
    Ok(Some(Self { clause, body }))
  }
}

impl MakeAst for DoWhile {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(TokenEnum::Keyword(Keyword::Do)) = stream.next_variant() else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let Some(body) = stream.make()? else {
      return ExpectedSnafu {
        what: "a block expression",
        span: stream.span()
      }.fail();
    };

    stream.skip_whitespace_and_comments();

    let Some(TokenEnum::Keyword(Keyword::While)) = stream.next_variant() else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let Some(clause) = stream.make()? else {
      return ExpectedSnafu {
        what: "an expression",
        span: stream.span()
      }.fail();
    };
    
    Ok(Some(Self { clause, body }))
  }
}

impl MakeAst for Until {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(TokenEnum::Keyword(Keyword::Until)) = stream.next_variant() else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let Some(clause) = stream.make()? else {
      return ExpectedSnafu {
        what: "an expression",
        span: stream.span()
      }.fail();
    };

    stream.skip_whitespace_and_comments();

    let Some(body) = stream.make()? else {
      return ExpectedSnafu {
        what: "a block expression",
        span: stream.span()
      }.fail();
    };
    
    Ok(Some(Self { clause, body }))
  }
}

impl MakeAst for DoUntil {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(TokenEnum::Keyword(Keyword::Do)) = stream.next_variant() else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let Some(body) = stream.make()? else {
      return ExpectedSnafu {
        what: "a block expression",
        span: stream.span()
      }.fail();
    };

    stream.skip_whitespace_and_comments();

    let Some(TokenEnum::Keyword(Keyword::Until)) = stream.next_variant() else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let Some(clause) = stream.make()? else {
      return ExpectedSnafu {
        what: "an expression",
        span: stream.span()
      }.fail();
    };
    
    Ok(Some(Self { clause, body }))
  }
}

impl MakeAst for For {
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
        Some(Self::If(r#if))
      } else if let Some(r#while) = stream.make()? {
        Some(Self::While(r#while))
      } else if let Some(do_while) = stream.make()? {
        Some(Self::DoWhile(do_while))
      } else if let Some(until) = stream.make()? {
        Some(Self::Until(until))
      } else if let Some(do_until) = stream.make()? {
        Some(Self::DoUntil(do_until))
      } else if let Some(r#for) = stream.make()? {
        Some(Self::For(r#for))
      } else if let Some(r#loop) = stream.make()? {
        Some(Self::Loop(r#loop))
      } else {
        None
      }
    })
  }  
}