use std::path::PathBuf;

use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError,
};

use crate::asterizer::error::ExpectedSnafu;

use crate::tokenizer::{
  Grouping,
  GroupingType,
  Keyword,
  Literal,
  Operator,
  Punctuation,
  Span,
  GetSpan,
  TokenEnum,
};

use crate::compiler::{
  Handle,
  SourceFile,
};

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct Qualify {
  pub(crate) name: String,
  pub(crate) rest: Box<ImportPattern>,
  pub(crate) span: Span,
}

#[derive(Debug, TypeName)]
pub(crate) enum ImportPattern {
  Qualify(Qualify),
  Tail(String),
  Wildcard,
  Brace(ImportBrace),
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct ImportBrace {
  pub(crate) children: Vec<ImportPattern>,
  pub(crate) span: Span,
}

#[derive(Debug, TypeName)]
pub(crate) enum TopLevelImportPattern {
  All(String),
  Brace(ImportBrace),
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct Import {
  pub(crate) pattern: TopLevelImportPattern,
  pub(crate) from: Handle,
  pub(crate) span: Span,
}

impl GetSpan for Qualify {
  fn get_span(&self) -> &Span {
    &self.span
  }
}

impl GetSpan for ImportPattern {
  fn get_span(&self) -> &Span {
    todo!()
  }
}

impl GetSpan for ImportBrace {
  fn get_span(&self) -> &Span {
    &self.span
  }
}

impl GetSpan for TopLevelImportPattern {
  fn get_span(&self) -> &Span {
    todo!()
  }
}

impl GetSpan for Import {
  fn get_span(&self) -> &Span {
    &self.span
  }
}

impl MakeAst for ImportPattern {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
        if let Some(TokenEnum::Identifier(name)) = stream.peek_variant() {
      let name = name.to_owned();
      stream.seek();

      stream.push_mark();
      stream.skip_whitespace_and_comments();

      if let Some(TokenEnum::Operator(Operator::Separator)) = stream.peek_variant() {
        stream.drop_mark();
        stream.seek();
        stream.skip_whitespace_and_comments();

        let Some(subpattern) = stream.make()? else {
          return ExpectedSnafu {
            what: "an import pattern",
            span: stream.span(),
          }.fail();
        };

        let rest = Box::new(subpattern);

        return Ok(Some(Self::Qualify(Qualify {
          name,
          rest,
          span: stream.span_mark(),
        })))
      };

      stream.pop_mark();

      return Ok(Some(Self::Tail(name)));
    };

    if let Some(TokenEnum::Operator(Operator::Multiply)) = stream.peek_variant() {
      stream.seek();

      return Ok(Some(Self::Wildcard));
    };

    let Some(brace) = stream.make()? else {
      return Ok(None);
    };

    Ok(Some(Self::Brace(brace)))
  }
}

impl MakeAst for ImportBrace {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(TokenEnum::Grouping(Grouping::Open(GroupingType::CurlyBrace))) = stream.next_variant() else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let mut children = vec![];
    loop {
      stream.skip_whitespace_and_comments();

      if let Some(TokenEnum::Grouping(Grouping::Close(GroupingType::CurlyBrace))) = stream.peek_variant() {
        break;
      };

      let Some(child) = stream.make()? else {
        return ExpectedSnafu {
          what: "an import pattern",
          span: stream.span(),
        }.fail();
      };

      children.push(child);

      stream.skip_whitespace_and_comments();

      if let Some(TokenEnum::Punctuation(Punctuation::Comma)) = stream.peek_variant() {
        stream.seek();
        continue;
      };

      let Some(TokenEnum::Grouping(Grouping::Close(GroupingType::CurlyBrace))) = stream.next_variant() else {
        return ExpectedSnafu {
          what: "a closing curly brace",
          span: stream.span(),
        }.fail();
      };

      break;
    };

    Ok(Some(Self {
      children,
      span: stream.span_mark(),
    }))
  }
}

impl MakeAst for TopLevelImportPattern {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
        if let Some(TokenEnum::Identifier(name)) = stream.peek_variant() {
      let name = name.to_owned();

      Ok(Some(Self::All(name)))
    } else if let Some(brace) = stream.make()? {
      Ok(Some(Self::Brace(brace)))
    } else {
      Ok(None)
    }
  }
}

impl MakeAst for Import {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(TokenEnum::Keyword(Keyword::Import)) = stream.next_variant() else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let Some(pattern) = stream.make()? else {
      return ExpectedSnafu {
        what: "an import pattern",
        span: stream.span(),
      }.fail();
    };

    stream.skip_whitespace_and_comments();

    let Some(TokenEnum::Keyword(Keyword::From)) = stream.next_variant() else {
      return ExpectedSnafu {
        what: "\"from\"",
        span: stream.span(),
      }.fail();
    };

    stream.skip_whitespace_and_comments();

    let Some(Literal::UnicodeString(relative_path)) = stream.make()? else {
      return ExpectedSnafu {
        what: "an import path",
        span: stream.span(),
      }.fail();
    };

    let mut relative_import_path: PathBuf = stream.path.to_owned().parent().expect("no parent directory").into();
    relative_import_path.push(relative_path);

    let from = stream.compiler.create_handle(SourceFile::new(relative_import_path));

    Ok(Some(Self {
      pattern,
      from,
      span: stream.span_mark(),
    }))
  }
}
