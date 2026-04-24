use lang::Compiler;
use ::lang::span::GetSpan;
use ::lang::token::{GroupingKind, GroupingType, Operator};
use crate::make::make_name;

use super::*;

pub(super) fn make_struct_initializer<'pool, C: Compiler, const N: usize, T: Read>(
  store: &mut C::Store<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  module: lang::ModuleReference,
  block: lang::BlockReference,
) -> Result<Option<lang::expr::Expression>, Error<C>> {
  let ret_mark = stream.mark();

  // Take care of the indentation; I have picked a very sketchy way of managing
  // these and it's not safe at all.  Too bad!
  let indenter = stream.indenter_here()?;
  // If I set up the indenter when the Indent(1..) comes, that means it can't
  // anything that happens after that block.  I'm wondering if struct
  // intializers even need to use braces, and instead perhaps a C-esque approach
  // would be more suitable:
  //
  // struct Foo
  //   u8 bar
  //   i32 baz
  //
  // main -> i32
  //
  //   // May need a special syntax to differentiate it from a variable assn.
  //   Foo obj [... extra syntax? ...]
  //     bar: 16
  //     baz: 256
  //
  //   obj.baz

  let Some(ty) = ty::make_type(store, stream, module)? else {
    return Ok(None);
  };
  let start = ty.get_span(store);

  stream.skip_whitespace_and_comments()?;

  let Some((Token::Grouping(GroupingType::Open(GroupingKind::Brace)), _)) = stream.peek()? else {
    // Do not forget to rewind `stream`!  You will look like a fool!
    stream.take_mark(ret_mark);

    // can possible something else, e.g. a variable's value
    return Ok(None);
  };
  stream.seek();

  stream.skip_whitespace_and_comments()?;

  let Some((Token::Indent(indent), indent_span)) = stream.peek()? else {
    return stream.expected_here(line_dbg!("an indent"));
  };
  stream.seek();

  match indent {
    // this is what we expect
    1.. => {},
    // no bueno
    ..=0 => return Err(Error::Invalid {
      what: line_dbg!("whitespace"),
      at: indent_span,
    }),
  };

  let mut members = vec![];

  loop {
    stream.skip_whitespace_and_comments()?;

    match indenter.peek(stream)? {
      // bad newline
      Some((Token::Indent(1..), at)) => {
        return Err(Error::Invalid {
          what: line_dbg!("indent"),
          at,
        });
      },
      // skip empty lines
      Some((Token::Indent(0), _)) => {
        stream.seek();
        continue;
      },
      // break out
      Some((Token::Indent(..=-1), _)) => {
        stream.seek();
        break;
      },
      // EOF or bad indentation
      None => {
        let _ = dbg!(stream.peek());
        return stream.expected_here(line_dbg!("a struct member or close brace; got EOF"));
      },
      // this'll mean code for us to parse
      Some(_) => {},
    };

    let Some(name) = make_name(stream)? else {
      return stream.expected_here(line_dbg!("a name"));
    };

    stream.skip_whitespace_and_comments()?;

    // there really should be a method to deal with this four-line-combo
    let Some((Token::Operator(Operator::Colon), _)) = indenter.peek(stream)? else {
      return stream.expected_here(line_dbg!("a colon"));
    };
    stream.seek();

    let Some(value) = make_expr(store, stream, module, block)? else {
      return stream.expected_here(line_dbg!("an expression"));
    };

    members.push((name, value));
  };

  stream.skip_whitespace_and_comments()?;

  let Some((Token::Grouping(GroupingType::Close(GroupingKind::Brace)), end)) = indenter.peek(stream)? else {
    return stream.expected_here(line_dbg!("a close brace"));
  };
  stream.seek();

  // todo!("do i skip the newline? {:#?}", indenter.peek(stream));

  let span = Span::from_pair(start, end);

  Ok(Some(lang::expr::Expression::StructInitializer {
    ty,
    members,
    span,
  }))
}
