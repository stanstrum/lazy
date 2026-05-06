use lang::reference::Store;
use lang::token::Operator;
use lang::span::GetSpan;
use lang::Compiler;
use lazy_macros::print_message;

use super::*;

type Value<C> = (lang::reference::VariableReference<C>, Option<lang::reference::ExpressionReference<C>>);

fn insert_variable_and_warn_conflicting_name<C: Compiler>(
  store: &mut C::Store<'_>,
  block_reference: lang::reference::BlockReference<C>,
  name: lang::module::Name<C>,
  type_value: lang::ty::TypeValue<C>,
  span: lang::span::Span<C>,
) -> lang::reference::VariableReference<C> {
  let function_span = block_reference.0.rget_from(store).span;
  let conflict = block::variable_resolution::resolve_variable(store, block_reference, &name.id);

  if let Some(conflict) = conflict {
    let conflict_span = (&*store).rget(conflict).span;

    print_message!(store, {
      level: Warn,
      force: false,
      description: line_dbg!("conflicting name will be shadowed").into(),
      contents: MessageContents::WithinSource(vec![WithinSource {
        range: function_span,
        sections: vec![
          MessageSection {
            text: "first used here".into(),
            span: conflict_span,
          },
          MessageSection {
            text: "shadowed here".into(),
            span: name.span,
          },
        ],
      }]),
    });
  };

  let block_borrow = store.rget_mut(block_reference);

  let variable_index = block_borrow.variables.len();
  let variable_reference = lang::reference::VariableReference::Block(block_reference, variable_index);

  let type_reference = lang::reference::TypeReference::Variable(variable_reference);
  let ty = lang::ty::Type::new(type_reference, type_value);

  let variable = lang::expr::Variable { name, ty, span };

  block_borrow.variables.push(variable);

  variable_reference
}

pub fn make_assignment<'pool, C: Compiler, const N: usize, T: Read>(
  store: &mut C::Store<'pool>,
  stream: &mut Rereader<'pool, C, N, T>,
  module: C::ModuleReference,
  block_reference: lang::reference::BlockReference<C>,
) -> Result<Option<Value<C>>, Error<C>> {
  let ret_mark = stream.mark();

  let Some(type_value) = ty::make_type(store, stream, module)? else {
    return Ok(None);
  };
  let mut span = type_value.get_span(store);

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

      let Some(expr) = make_expr(store, stream, module, block_reference)? else {
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

  let variable_reference = insert_variable_and_warn_conflicting_name(store,
    block_reference, name, type_value, span);

  Ok(Some((variable_reference, expr)))
}
