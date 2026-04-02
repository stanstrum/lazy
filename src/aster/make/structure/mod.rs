mod import;

use std::path::PathBuf;

use crate::aster::pprint::Pretty;
use crate::line_dbg;
use crate::error::WithinSource;
use crate::string_pool::StringId;

use crate::lang::reference::AliasReference;
use crate::lang::span::GetSpan;
use crate::tokenize::token::{Keyword, Operator};

use super::*;

#[derive(Debug)]
pub enum Structure {
  Function(lang::reference::FunctionReference),
  TypeAlias(lang::reference::AliasReference),
  ImportFrom(()),
}

fn make_type_alias<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  parent: lang::reference::ModuleReference,
) -> Result<Option<lang::reference::AliasReference>, Error> {
  let Some((Token::Keyword(Keyword::Type), start_span)) = stream.peek()? else {
    return Ok(None);
  };
  stream.seek();

  if !stream.skip_whitespace_and_comments()? {
    return stream.expected_here(line_dbg!("whitespace"));
  };

  let Some(name) = make_name(stream)? else {
    return stream.expected_here(line_dbg!("a name"));
  };

  stream.skip_whitespace_and_comments()?;

  let Some((Token::Operator(Operator::Bollocks), _)) = stream.peek()? else {
    return stream.expected_here(line_dbg!("assignment operator (:=)"));
  };
  stream.seek();

  stream.skip_whitespace_and_comments()?;

  let Some(ty) = ty::make_type(lazy, stream, parent)? else {
    return stream.expected_here(line_dbg!("a type"));
  };

  let mut span = start_span;
  span.extend(ty.get_span(lazy));

  // TODO: put this into a method
  let index = lazy.rget(parent).aliases.len();
  let alias_reference = AliasReference(parent, index);

  lazy.rget_mut(parent).aliases.push(lang::module::TypeAlias {
    name,
    span,
    ty,
  });

  Ok(Some(alias_reference))
}

pub(super) fn make_structure<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  parent: lang::reference::ModuleReference,
) -> Result<Option<Structure>, Error> {
  if let Some(function) = function::make_function(lazy, stream, parent)? {
    let name = &function.rget_from(lazy).header.name;
    let (name, span) = (
      lazy.pool.get(name.id).collect::<String>(),
      name.span,
    );

    let module_name = lazy.describe_module(stream.module);

    print_message(lazy, PrintableMessage {
      level: Level::Debug,
      force: false,
      description: format!(line_dbg!("parsed a function: {}::{}"), module_name, name),
      contents: MessageContents::WithinSource(vec![WithinSource {
        range: span,
        sections: vec![MessageSection {
          text: "here".into(),
          span,
        }],
      }]),
    });

    return Ok(Some(Structure::Function(function)))
  };

  if let Some(alias) = make_type_alias(lazy, stream, parent)? {
    let alias_ref = alias.rget_from(lazy);
    let name = lazy.pool.get(alias_ref.name.id).collect::<String>();

    let module_name = lazy.describe_module(stream.module);

    print_message(lazy, PrintableMessage {
      level: Level::Debug,
      force: false,
      description: format!(line_dbg!("parsed a type alias: {}::{}"), module_name, name),
      contents: MessageContents::WithinSource(vec![WithinSource {
        range: alias_ref.span,
        sections: vec![
          MessageSection {
            text: "here".into(),
            span: alias_ref.name.span,
          },
          MessageSection {
            text: "the type".into(),
            span: alias_ref.ty.get_span(lazy),
          },
        ],
      }]),
    });

    return Ok(Some(Structure::TypeAlias(alias)))
  };

  if let Some(import) = import::make_import(lazy, stream)? {
    something(lazy, &import);

    todo!("import:#?")
  };

  Ok(None)
}

fn print_names(lazy: &lang::Lazy, names: &[lang::module::Name], end_asterisk: bool, out: &mut Vec<lang::ty::Qualified>) {
  let mut string = String::new();

  for part in names {
    let name = lazy.pool.get(part.id).collect::<String>();

    string += &format!("::{name}");
  };

  if end_asterisk {
    string += "::*";
  };

  println!("{}", string);
  out.push(lang::ty::Qualified {
    implicit: lang::ty::QualifiedSearchSpace::Implicit,
    parts: names.to_owned(),
    span: names.last().unwrap().span,
  });
}

fn part(lazy: &lang::Lazy, selector: &import::ImportPart, names: &mut Vec<lang::module::Name>, out: &mut Vec<lang::ty::Qualified>) {
  match selector {
    import::ImportPart::Star(_) => {
      print_names(lazy, names, true, out);
    },
    import::ImportPart::Group(import_group) => group(lazy, import_group, names, out),
    import::ImportPart::Qualify(import_qualify) => {
      names.push(import_qualify.name);

      if let Some(next) = &import_qualify.next {
        part(lazy, next, names, out);
      } else {
        print_names(lazy, names, false, out);
      };

      names.pop();
    },
  }
}

fn group(lazy: &lang::Lazy, group: &import::ImportGroup, names: &mut Vec<lang::module::Name>, out: &mut Vec<lang::ty::Qualified>) {
  for selector in group.selectors.iter() {
    part(lazy, selector, names, out);
  };
}

fn something(lazy: &lang::Lazy, import: &import::Import) {
  let mut out = vec![];
  let mut names = vec![];

  // lang::ty::Qualified {
  //   implicit: lang::ty::QualifiedSearchSpace::Module(added_module),
  //   parts: todo!(),
  //   span: todo!(),
  // };

  group(lazy, &import.group, &mut names, &mut out);

  for out in out {
    println!("16: {}", out.print(lazy));
  };

  todo!()
}
