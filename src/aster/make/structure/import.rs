use crate::tokenize::token::StringKind;

use super::*;

#[derive(Debug)]
enum ImportPart {
  Star(Span),
  Name(lang::module::Name),
  Group {
    selectors: Vec<ImportPart>,
    span: Span,
  },
  Qualify {
    name: lang::module::Name,
    next: Box<ImportPart>,
    span: Span,
  },
}

#[derive(Debug)]
pub(super) struct Import {
  source: StringId,
  selectors: Vec<ImportPart>,
}

fn make_selector<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  indenter: &Indenter,
) -> Result<Option<ImportPart>, Error> {
  todo!()
}

pub(super) fn make_import<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
) -> Result<Option<Import>, Error> {
  let Some((Token::Keyword(Keyword::Import), start)) = stream.peek()? else {
    return Ok(None);
  };
  stream.seek();
  stream.skip_whitespace_and_comments()?;

  let Some((Token::Keyword(Keyword::From), _)) = stream.peek()? else {
    return stream.expected_here(line_dbg!("keyword (from)"));
  };
  stream.seek();
  stream.skip_whitespace_and_comments()?;

  let Some(expr) = expr::make_literal(lazy, stream)? else {
    return stream.expected_here(line_dbg!("the path literal"));
  };

  let lang::expr::Expression::Literal { value, span, out } = expr else {
    return Err(Error::Invalid {
      what: line_dbg!("expression, expected string literal"),
      at: expr.get_span(lazy),
    });
  };

  let lang::expr::LiteralKind::String { value, kind: StringKind::Wide } = value else {
    return Err(Error::Invalid {
      what: line_dbg!("literal, expected normal string"),
      at: span,
    });
  };

  stream.skip_whitespace_and_comments()?;

  let mut import = Import {
    source: value,
    selectors: vec![]
  };

  let Some((Token::Indent(indent), indent_span)) = stream.peek()? else {
    return stream.expected_here(line_dbg!("a positive indent"));
  };

  if indent <= 0 {
    return Ok(Some(import))
  };

  let indenter = stream.indenter_here()?;

  while let Some(selector) = make_selector(lazy, stream, &indenter)? {
    import.selectors.push(selector);
  };

  Ok(Some(import))
}
