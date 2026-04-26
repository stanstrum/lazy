use lang::Compiler;
use lang::token::Operator;
use lang::span::GetSpan;

use super::*;

type Value<C> = (lang::expr::Variable<C>, Option<lang::reference::ExpressionReference<C>>);

pub fn make_assignment<'pool, C: Compiler, const N: usize, T: Read>(
  store: &mut C::Store<'pool>,
  stream: &mut Rereader<'pool, C, N, T>,
  module: C::ModuleReference,
  block: BlockReference<C>,
) -> Result<Option<Value<C>>, Error<C>> {
  let ret_mark = stream.mark();

  let Some(ty) = ty::make_type(store, stream, module)? else {
    return Ok(None);
  };
  let mut span = ty.get_span(store);

  stream.skip_whitespace_and_comments()?;

  let Some(name) = crate::make::make_name(stream)? else {
    stream.take_mark(ret_mark);
    return Ok(None);
  };

  let pre_assignment_mark = stream.mark();
  stream.skip_whitespace_and_comments()?;

  let expr = {
    if let Some((Token::Operator(Operator::Bollocks), _)) = stream.peek()? {
      stream.seek();
      stream.skip_whitespace_and_comments()?;

      let Some(expr) = make_expr(store, stream, module, block)? else {
        return stream.expected_here(line_dbg!("an expression"));
      };

      span.extend(expr.rget_from(store).get_span(store));

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
