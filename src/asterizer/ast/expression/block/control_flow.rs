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

use crate::asterizer::error::*;

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct IfBranch {
  pub(crate) clause: Expression,
  pub(crate) body: Block,
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct If {
  pub(crate) branches: Vec<IfBranch>,
  pub(crate) r#else: Option<Block>,
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct While {
  pub(crate) clause: Expression,
  pub(crate) body: Block,
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct DoWhile {
  pub(crate) body: Block,
  pub(crate) clause: Expression,
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct Until {
  pub(crate) clause: Expression,
  pub(crate) body: Block,
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct DoUntil {
  pub(crate) body: Block,
  pub(crate) clause: Expression,
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct For {
  pub(crate) clause: Expression,
  pub(crate) body: Block,
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct Loop {
  pub(crate) body: Block,
}

#[allow(unused)]
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

impl MakeAst for IfBranch {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(TokenEnum::Keyword(Keyword::If)) = stream.next_variant() else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let Some(clause) = stream.make()? else {
      return ExpectedSnafu {
        what: "an expression",
        span: stream.span(),
      }.fail();
    };

    stream.skip_whitespace_and_comments();

    let Some(body) = stream.make()? else {
      return ExpectedSnafu {
        what: "an expression",
        span: stream.span(),
      }.fail();
    };

    Ok(Some(Self { clause, body }))
  }
}

impl MakeAst for If {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(TokenEnum::Keyword(Keyword::If)) = stream.peek_variant() else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let Some(primary) = stream.make()? else {
      return ExpectedSnafu {
        what: "an if branch",
        span: stream.span(),
      }.fail();
    };

    let mut branches = vec![primary];
    let mut r#else = None;

    loop {
      stream.push_mark();
      stream.skip_whitespace_and_comments();

      let Some(TokenEnum::Keyword(Keyword::Else)) = stream.next_variant() else {
        stream.pop_mark();

        break;
      };

      stream.drop_mark();
      stream.skip_whitespace_and_comments();

      if let Some(branch) = stream.make()? {
        branches.push(branch);

        continue;
      };

      r#else = stream.make()?;

      if r#else.is_none() {
        return ExpectedSnafu {
          what: "an else block",
          span: stream.span(),
        }.fail();
      };

      break;
    };

    Ok(Some(Self { branches, r#else }))
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
        span: stream.span(),
      }.fail();
    };

    stream.skip_whitespace_and_comments();

    let Some(body) = stream.make()? else {
      return ExpectedSnafu {
        what: "a block expression",
        span: stream.span(),
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
        span: stream.span(),
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
        span: stream.span(),
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
        span: stream.span(),
      }.fail();
    };

    stream.skip_whitespace_and_comments();

    let Some(body) = stream.make()? else {
      return ExpectedSnafu {
        what: "a block expression",
        span: stream.span(),
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
        span: stream.span(),
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
        span: stream.span(),
      }.fail();
    };

    Ok(Some(Self { clause, body }))
  }
}

impl MakeAst for For {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(TokenEnum::Keyword(Keyword::For)) = stream.peek_variant() else {
      return Ok(None)
    };

    NotImplementedSnafu {
      message: "for loops are not yet implemented",
      span: stream.span(),
    }.fail()
  }
}

impl MakeAst for Loop {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(TokenEnum::Keyword(Keyword::Loop)) = stream.next_variant() else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let Some(body) = stream.make()? else {
      return ExpectedSnafu {
        what: "a block expression",
        span: stream.span(),
      }.fail();
    };

    Ok(Some(Self { body }))
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
