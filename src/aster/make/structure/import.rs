use crate::tokenize::token::StringKind;
use crate::print_once_per_thread;

use super::*;

#[derive(Debug)]
pub(super) struct ImportGroup {
  pub(super) selectors: Vec<ImportPart>,
  pub(super) span: Span,
}

#[derive(Debug)]
pub(super) struct ImportQualify {
  pub(super) name: lang::module::Name,
  pub(super) next: Option<Box<ImportPart>>,
  pub(super) span: Span,
}

#[derive(Debug)]
pub(super) enum ImportPart {
  Star(Span),
  Group(ImportGroup),
  Qualify(ImportQualify),
}

impl ImportQualify {
  fn new(name: lang::module::Name) -> Self {
    Self {
      name,
      next: None,
      span: name.span,
    }
  }
}

fn print_part(lazy: &lang::Lazy, part: &ImportPart, indent: usize, out: &mut String) {
  match part {
    ImportPart::Star(_) => *out += "*",
    ImportPart::Group(import_group) => {
      print_group(lazy, import_group, indent + 1, out);
    },
    ImportPart::Qualify(import_qualify) => {
      let name = lazy.pool.get(import_qualify.name.id)
        .collect::<String>();
      *out += &name;

      if let Some(next) = &import_qualify.next {
        *out += "::";

        print_part(lazy, next, indent, out);
      };
    },
  };
}

fn print_group(lazy: &lang::Lazy, group: &ImportGroup, indent: usize, out: &mut String) {
  let padding = " ".repeat(2 * indent);

  for selector in group.selectors.iter() {
    *out += &format!("\n{padding}");

    print_part(lazy, selector, indent, out);
  };
}

fn print_import(lazy: &lang::Lazy, import: &Import) -> String {
  let w = lazy.pool.get_string(import.source);

  let mut out = format!("import from {w:?}");

  print_group(lazy, &import.group, 1, &mut out);

  out
}

impl GetSpan for ImportPart {
  fn get_span(&self, lazy: &lang::Lazy) -> Span {
    match self {
      ImportPart::Star(span) => *span,
      ImportPart::Group(group) => group.span,
      ImportPart::Qualify(qualify) => qualify.span,
    }
  }
}

#[derive(Debug)]
pub(super) struct Import {
  pub source: StringId,
  pub group: ImportGroup,
}

fn make_group<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  indenter: &Indenter,
) -> Result<Option<ImportGroup>, Error> {
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

  Ok(Some(ImportGroup {
    selectors,
    span,
  }))
}

fn make_qualify<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  indenter: &Indenter,
) -> Result<Option<ImportQualify>, Error> {
  let Some((Token::Identifier(_), _)) = indenter.peek(stream)? else {
    return Ok(None);
  };

  let name = make_name(stream)?.unwrap();

  stream.skip_whitespace_and_comments()?;

  let mut qualify = ImportQualify::new(name);

  if let Some((Token::Operator(Operator::DoubleColon), colon)) = indenter.peek(stream)? {
    stream.seek();
    stream.skip_whitespace_and_comments()?;

    qualify.span.extend(colon);

    let Some(next) = make_selector(lazy, stream, &indenter)?.map(Box::new) else {
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
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  indenter: &Indenter,
) -> Result<Option<ImportPart>, Error> {
  if let Some(qualify) = make_qualify(lazy, stream, indenter)? {
    return Ok(Some(ImportPart::Qualify(qualify)))
  };

  if let Some(group) = make_group(lazy, stream, indenter)? {
    return Ok(Some(ImportPart::Group(group)))
  };

  if let Some((Token::Operator(Operator::Asterisk), span)) = indenter.peek(stream)? {
    stream.seek();
    stream.skip_whitespace_and_comments()?;

    if let Some((Token::Indent(_), _)) = stream.peek()? {
      stream.seek();
    };

    return Ok(Some(ImportPart::Star(span)));
  };

  print_once_per_thread!(lazy, {
    level: Level::Stub,
    force: false,
    description: line_dbg!("parse other kinds of selector").into(),
    contents: MessageContents::None,
  });

  Ok(None)
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

  let indenter = stream.indenter_here()?;
  let Some(group) = make_group(lazy, stream, &indenter)? else {
    return stream.expected_here(line_dbg!("an import group"));
  };

  let import = Import {
    source: value,
    group,
  };

  println!("{}", print_import(lazy, &import));

  Ok(Some(import))
}
