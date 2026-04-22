use crate::tokenize::token::Operator;
use crate::lang::span::GetSpan;
use crate::aster::make::{make_name, ty};

use super::*;

type Value = (lang::expr::Variable, Option<lang::reference::ExpressionReference>);

pub fn make_assignment<'pool, const N: usize, T: Read>(
  lazy: &mut lazy::Lazy<'pool>,
  stream: &mut Rereader<'pool, N, T>,
  module: lang::reference::ModuleReference,
  block: lang::reference::BlockReference,
) -> Result<Option<Value>, Error> {
  let ret_mark = stream.mark();

  let Some(ty) = ty::make_type(lazy, stream, module)? else {
    return Ok(None);
  };
  let mut span = ty.get_span(lazy);

  stream.skip_whitespace_and_comments()?;

  let Some(name) = make_name(stream)? else {
    stream.take_mark(ret_mark);
    return Ok(None);
  };

  let pre_assignment_mark = stream.mark();
  stream.skip_whitespace_and_comments()?;

  let expr = {
    if let Some((Token::Operator(Operator::Bollocks), _)) = stream.peek()? {
      stream.seek();
      stream.skip_whitespace_and_comments()?;

      let Some(expr) = make_expr(lazy, stream, module, block)? else {
        return stream.expected_here(line_dbg!("an expression"));
      };

      span.extend(expr.rget_from(lazy).get_span(lazy));

      Some(expr)
    } else {
      stream.take_mark(pre_assignment_mark);
      span.extend(name.span);

      None
    }
  };

  let variable = lang::expr::Variable { name, ty, span };

  Ok(Some((variable, expr)))
}
