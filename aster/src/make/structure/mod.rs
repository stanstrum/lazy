mod module;
mod import;
mod traverser;

use crate::{line_dbg, print_message};

use ::lang::span::GetSpan;
use crate::lang::AliasReference;
use ::lang::token::{Keyword, Operator};

use super::*;

#[derive(Debug)]
pub enum Structure {
  Module(lang::ModuleReference),
  Function(lang::FunctionReference),
  TypeAlias(lang::AliasReference),
  Struct(lang::StructReference),
  ImportFrom(()),
}

fn make_type_alias<'pool, const N: usize, T: Read>(
  lazy: &mut crate::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  parent: lang::ModuleReference,
) -> Result<Option<lang::AliasReference>, Error> {
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

/// Not a fan of the similarity between [`make_struct`] and [`make_structure`],
/// in their names.  This one is for making [`lang::module::struc::Struct`]s:
///
///     struct Square
///       u32 height
///       u32 width
fn make_struct<'pool, const N: usize, T: Read>(
  lazy: &mut crate::Lazy<'pool>,
  parent: lang::ModuleReference,
  stream: &mut Rereader<'pool, N, T>,
) -> Result<Option<lang::module::struc::Struct>, Error> {
  let Some((Token::Keyword(Keyword::Struct), start)) = stream.peek()? else {
    return Ok(None);
  };

  let indenter = stream.indenter_here()?;
  stream.seek();

  if !stream.skip_whitespace_and_comments()? {
    return stream.expected_here(line_dbg!("whitespace"));
  };

  let Some(name) = make_name(stream)? else {
    return stream.expected_here(line_dbg!("a name"));
  };

  stream.skip_whitespace_and_comments()?;

  let mut members = vec![];

  match indenter.peek(stream)? {
    Some((Token::Indent(1..), _)) => {
      stream.seek();
    },
    Some((Token::Indent(0), _)) | None => {
      return Ok(Some(lang::module::struc::Struct {
        name,
        members,
        span: Span::from_pair(start, stream.here()?),
      }));
    },
    other => todo!("{other:#?}"),
  };

  let end;
  loop {
    stream.skip_whitespace_and_comments()?;

    match indenter.peek(stream)? {
      Some((Token::Indent(1..), at)) => {
        return Err(Error::Invalid {
          what: line_dbg!("indent"),
          at,
        });
      },
      Some((Token::Indent(0), _)) => {
        // skip empty lines
        stream.seek();
        continue;
      },
      next_tok @ (None | Some((Token::Indent(..=-1), _))) => {
        end = stream.here()?;
        // only skip the token if we can see it.  TODO: would be nice to have
        // some kind of a shorthand for this
        if next_tok.is_some() { stream.seek(); };
        break;
      },
      Some(_) => {
        // this will be code for us to
      },
    };

    let Some(variable) = function::make_function_argument(lazy, stream, parent)? else {
      return stream.expected_here(line_dbg!("a struct member"));
    };

    members.push(variable);
  };

  let span = Span::from_pair(start, end);

  Ok(Some(lang::module::struc::Struct {
    name,
    members,
    span,
  }))
}

/// This should be the entry point to making a structure, whether from top-level
/// or from within a submodule, since this is where the processing/registration
/// of the data structues get handled, i.e. storing the module in `parent` or
/// traversing imports.
pub(super) fn make_structure<'pool, const N: usize, T: Read>(
  lazy: &mut crate::Lazy<'pool>,
  parent: lang::ModuleReference,
  stream: &mut Rereader<'pool, N, T>,
) -> Result<Option<Structure>, Error> {
  let here = stream.here()?;

  if let Some(module) = module::make_mod(lazy, stream, parent)? {
    lazy.rget_mut(parent).modules.push(module);

    let module_name = lazy.describe_module(module);

    print_message!(lazy, {
      level: Debug,
      force: false,
      description: format!(line_dbg!("parsed a module: {}"), module_name),
      contents: MessageContents::WithinSource(vec![WithinSource {
        range: here,
        sections: vec![MessageSection {
          text: "here".into(),
          span: here,
        }],
      }]),
    });

    return Ok(Some(Structure::Module(module)));
  };

  if let Some(function) = function::make_function(lazy, stream, parent)? {
    let name = &function.rget_from(lazy).header.name;
    let (name, span) = (
      lazy.pool.get(name.id),
      name.span,
    );

    let module_name = lazy.describe_module(parent);

    print_message!(lazy, {
      level: Debug,
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
    let name = lazy.pool.get(alias_ref.name.id);

    let module_name = lazy.describe_module(parent);

    print_message!(lazy, {
      level: Debug,
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

  if let Some(struc) = make_struct(lazy, parent, stream)? {
    // Make the next StructReference for the struct and then add it.
    // TODO: This is far too clumsy to keep this way forever
    let parent_borrow = lazy.rget_mut(parent);
    let id = parent_borrow.structs.len();
    let struct_reference = lang::StructReference(parent, id);
    parent_borrow.structs.push(struc);
    return Ok(Some(Structure::Struct(struct_reference)))
  };

  if let Some(import) = import::make_import(lazy, parent, stream)? {
    traverser::traverse_import(lazy, stream.module, &import)?;

    return Ok(Some(Structure::ImportFrom(())));
  };

  Ok(None)
}
