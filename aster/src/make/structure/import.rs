use std::path::PathBuf;

use crate::print_once_per_thread;
use ::lang::token::StringKind;

use super::*;

fn make_group<'pool, const N: usize, T: Read>(
  lazy: &mut crate::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  indenter: &Indenter,
) -> Result<Option<crate::lang::module::import::ImportGroup>, Error> {
  let Some((Token::Indent(indent), mut span)) = indenter.peek(stream)? else {
    return Ok(None);
  };

  if indent <= 0 {
    return Ok(None);
  };

  stream.seek();
  stream.skip_whitespace_and_comments()?;

  let indenter = stream.indenter_here()?;
  let mut selectors = vec![];

  loop {
    stream.skip_whitespace_and_comments()?;

    if let Some((Token::Indent(0), _)) = indenter.peek(stream)? {
      stream.seek();
      continue;
    };

    let Some(selector) = make_selector(lazy, stream, &indenter)? else {
      break;
    };

    selectors.push(selector);
  };

  if let Some(last) = selectors.last() {
    span.extend(last.get_span(lazy));
  };

  Ok(Some(lang::module::import::ImportGroup {
    selectors,
    span,
  }))
}

fn make_qualify<'pool, const N: usize, T: Read>(
  lazy: &mut crate::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  indenter: &Indenter,
) -> Result<Option<lang::module::import::ImportQualify>, Error> {
  let Some((Token::Identifier(_), _)) = indenter.peek(stream)? else {
    return Ok(None);
  };

  let name = make_name(stream)?.unwrap();

  stream.skip_whitespace_and_comments()?;

  let mut qualify = lang::module::import::ImportQualify::new(name);

  if let Some((Token::Operator(Operator::DoubleColon), colon)) = indenter.peek(stream)? {
    stream.seek();
    stream.skip_whitespace_and_comments()?;

    qualify.span.extend(colon);

    let Some(next) = make_selector(lazy, stream, indenter)?.map(Box::new) else {
      return stream.expected_here(line_dbg!("a qualification"))
    };

    qualify.span.extend(next.get_span(lazy));

    qualify.next = Some(next);
  } else {
    match stream.peek()? {
      None => {},
      Some((Token::Indent(..=0), _)) => stream.seek(),
      _ => return stream.expected_here(line_dbg!("an indent (negative or zero)")),
    };
  };

  Ok(Some(qualify))
}

fn make_selector<'pool, const N: usize, T: Read>(
  lazy: &mut crate::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  indenter: &Indenter,
) -> Result<Option<lang::module::import::ImportPart>, Error> {
  if let Some(qualify) = make_qualify(lazy, stream, indenter)? {
    return Ok(Some(lang::module::import::ImportPart::Qualify(qualify)))
  };

  if let Some(group) = make_group(lazy, stream, indenter)? {
    return Ok(Some(lang::module::import::ImportPart::Group(group)))
  };

  if let Some((Token::Operator(Operator::Asterisk), span)) = indenter.peek(stream)? {
    stream.seek();
    stream.skip_whitespace_and_comments()?;

    if let Some((Token::Indent(_), _)) = stream.peek()? {
      stream.seek();
    };

    return Ok(Some(lang::module::import::ImportPart::Star(span)));
  };

  print_once_per_thread!(lazy, {
    level: Stub,
    force: false,
    description: line_dbg!("parse other kinds of selector").into(),
    contents: MessageContents::None,
  });

  Ok(None)
}

pub(super) fn make_import<'pool, const N: usize, T: Read>(
  lazy: &mut crate::Lazy<'pool>,
  module: lang::ModuleReference,
  stream: &mut Rereader<'pool, N, T>,
) -> Result<Option<lang::module::import::Import>, Error> {
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

  let lang::expr::Expression::Literal { value, span: literal_span, .. } = expr else {
    return Err(Error::Invalid {
      what: line_dbg!("expression, expected string literal"),
      at: expr.get_span(lazy),
    });
  };

  let lang::expr::LiteralKind::String { value, kind: StringKind::Wide } = value else {
    return Err(Error::Invalid {
      what: line_dbg!("literal, expected normal string"),
      at: literal_span,
    });
  };

  stream.skip_whitespace_and_comments()?;

  let indenter = stream.indenter_here()?;
  let Some(group) = make_group(lazy, stream, &indenter)? else {
    return stream.expected_here(line_dbg!("an import group"));
  };

  let end = group.span;
  let span = Span::from_pair(start, end);

  // Set up the imported source file's name, path
  let name = lazy.pool.get_own_string(value);
  let path = PathBuf::from(&name);

  // This is where we're going to look for this file if its path is relative:
  // in the directory of the current module
  let current_path = lazy.get_path(module).path.as_path();
  let relative_to = current_path.parent()
    .expect("source to have a parent directory")
    .to_owned();

  let source = lazy.add_file(&name, path, Some(&relative_to))?;

  Ok(Some(lang::module::import::Import {
    source,
    group,
    span,
  }))
}
