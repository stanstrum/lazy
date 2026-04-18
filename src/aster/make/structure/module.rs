use super::*;

pub(super) fn make_mod<'pool, const N: usize, T: Read>(
  lazy: &mut lang::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  parent: lang::reference::ModuleReference,
) -> Result<Option<lang::reference::ModuleReference>, Error> {
  let Some((Token::Keyword(Keyword::Mod), _)) = stream.peek()? else {
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

  // set up the module, even if it'll be empty
  let name_value = lazy.pool.get(name.id);
  let module = lazy.create_module(&name_value, |_, _| lang::module::ModuleParent::Module(parent));

  stream.skip_whitespace_and_comments()?;

  // deal with the indent
  match indenter.peek(stream)? {
    Some((Token::Indent(1..), _)) => {
      // continue on reading the body
      stream.seek();
    },
    other => todo!("{other:#?}"),
  };

  loop {
    stream.skip_whitespace_and_comments()?;

    // clean up blank lines, comments, etc
    match indenter.peek(stream)? {
      // de-indent?
      Some((Token::Indent(..=-1), _)) => {
        stream.seek();
        break;
      },
      // EOF or the indenter telling us to give it up
      None => todo!(),
      // skip a blank line
      Some((Token::Indent(0), _)) => {
        stream.seek();
        continue;
      },
      // unexpected indent
      Some((Token::Indent(1..), at)) => return Err(Error::Invalid {
        what: line_dbg!("positive indent"),
        at,
      }),
      // anything else is code for us to parse
      Some(_) => {},
    };

    // build a structure inside of `module`, not `parent`
    let Some(_) = structure::make_structure(lazy, module, stream)? else {
      return stream.expected_here(line_dbg!("a structure"));
    };
  };

  Ok(Some(module))
}
