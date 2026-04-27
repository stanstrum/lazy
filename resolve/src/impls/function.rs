use lazy_macros::{print_message, print_once_per_thread};

use lang::{Compiler, CompilerPoolStore};
use lang::reference::{ExpressionReference, TypeReference, VariableReference};
use lang::ty::{Type, TypePair};

use super::*;

pub(in crate::impls) fn resolve_function_reference<C: Compiler + 'static>(
  store: &C::Store<'_>,
  function_reference: &C::FunctionReference,
  tasks: &mut Tasks<C>,
) -> Result<C> {
  let description = format!(line_dbg!("Resolve FunctionReference: {}"), pprint::print_function_reference::<C>(function_reference, store));

  tasks.work(description, |tasks|{
    let function = function_reference.rget_from(store);
    let ret_ty = TypeReference::ReturnTypeOf(*function_reference);

    ret_ty.resolve(store, tasks)?;

    let arguments_iter = (0..function.header.arguments.len())
      .map(|index| TypeReference::Variable(VariableReference::Argument(*function_reference, index)));

    for argument in arguments_iter {
      argument.resolve(store, tasks)?;
    };

    let body = store.rget(function.body);
    if let Some(ty) = ret_ty.type_of(store) {
      let expr_id = body.children.last().unwrap();
      let reference = TypeReference::Expression(ExpressionReference(function.body, *expr_id));
      let typed_reference = Type::Reference(reference);

      let last_expression = TypePair::new(reference, typed_reference);
      let return_type: TypePair<C> = TypePair::new(ret_ty, ty);

      last_expression.coerce(store, &return_type, tasks)?;
    };

    function.body.resolve(store, tasks)?;

    Ok(())
  })
}

pub(super) fn default_types_in_function<C: Compiler + 'static>(
  store: &mut C::Store<'_>,
  function: &C::FunctionReference,
  tasks: &mut Tasks<C>,
) -> Result<C> {
  let header_arguments;
  let body;

  let description = {
    let borrow = function.rget_from(store);

    header_arguments = borrow.header.arguments.len();
    body = borrow.body;

    let parent = store.describe_module(borrow.parent);
    let name = store.pool().get(borrow.header.name.id);

    format!(line_dbg!("Make default ambiguous types in function: {}::{}"), parent, name)
  };

  tasks.work(
    description,
    |tasks| {
      ty::default_types_of_type(store,
        &TypeReference::ReturnTypeOf(*function),
        tasks,
      )?;

      for i in 0..header_arguments {
        ty::default_types_of_type(store,
          &TypeReference::Variable(VariableReference::Argument(*function, i)),
          tasks,
        )?
      };

      expr::default_types_in_block_expr(store, &body, tasks)?;

      Ok(())
    },
  )
}

pub(crate) fn verify_function<C: Compiler + 'static>(
  store: &C::Store<'_>,
  function: &C::FunctionReference,
  tasks: &mut Tasks<C>,
) -> Result<C> {
  // get main function
  let borrow = function.rget_from(store);

  let parent = store.describe_module(borrow.parent);
  let name = store.pool().get(borrow.header.name.id);

  tasks.work(
    format!(line_dbg!("Verify function: {}::{}"), parent, name),
    |tasks| {
    let ret_ty_reference = TypeReference::ReturnTypeOf(*function);

    // verify return type
    let ret_ty_pair = tasks.work(line_dbg!("verify return type").into(),
    |tasks| -> Result<C, TypePair<C>> {
        let ret_ty = &borrow.header.ret_ty;
        ty::verify_type(store, ret_ty, tasks)?;

        Ok(TypePair::new(
          ret_ty_reference,
          ret_ty.clone(),
        ))
      },
    )?;

    // verify argument types
    for argument in borrow.header.arguments.iter() {
      ty::verify_type(store, &argument.ty, tasks)?;
    };

    {
      let root = store.get_root_module(borrow.parent);

      print_once_per_thread!(store, {
        level: Stub,
        force: false,
        description: line_dbg!("verify that main arguments match expected function signature").into(),
        contents: MessageContents::File::<C>(root),
      });
    };

    // verify body
    expr::verify_block(store, &borrow.body, Some(&ret_ty_pair), tasks)?;

    Ok(())
  })
}
