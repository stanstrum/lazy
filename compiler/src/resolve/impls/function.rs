use crate::{print_message, print_once_per_thread};

use crate::resolve::TypePair;
use crate::lang::reference::{FunctionReference, TypeReference, VariableReference};

use super::*;

pub(super) fn default_types_in_function(lazy: &mut Lazy, function: &FunctionReference, tasks: &mut Tasks) -> Result<()> {
  let header_arguments;
  let body;

  let description = {
    let borrow = function.rget_from(lazy);

    header_arguments = borrow.header.arguments.len();
    body = borrow.body;

    let parent = lazy.describe_module(borrow.parent);
    let name = lazy.pool.get(borrow.header.name.id);

    format!(line_dbg!("Make default ambiguous types in function: {}::{}"), parent, name)
  };

  tasks.work(
    description,
    |tasks| {
      ty::default_types_of_type(lazy,
        &TypeReference::ReturnTypeOf(*function),
        tasks,
      )?;

      for i in 0..header_arguments {
        ty::default_types_of_type(lazy,
          &TypeReference::Variable(VariableReference::Argument(*function, i)),
          tasks,
        )?
      };

      expr::default_types_in_block_expr(lazy, &body, tasks)?;

      Ok(())
    },
  )
}

pub(in crate::resolve) fn verify_function(lazy: &Lazy, function: &FunctionReference, tasks: &mut Tasks) -> Result<()> {
  // get main function
  let borrow = function.rget_from(lazy);

  let parent = lazy.describe_module(borrow.parent);
  let name = lazy.pool.get(borrow.header.name.id);

  tasks.work(
    format!(line_dbg!("Verify function: {}::{}"), parent, name),
    |tasks| {
    let ret_ty_reference = TypeReference::ReturnTypeOf(*function);

    // verify return type
    let ret_ty_pair = tasks.work(line_dbg!("verify return type").into(),
    |tasks| -> Result<TypePair> {
        let ret_ty = &borrow.header.ret_ty;
        ty::verify_type(lazy, ret_ty, tasks)?;

        Ok(TypePair::new(
          ret_ty_reference,
          ret_ty.clone(),
        ))
      },
    )?;

    // verify argument types
    for argument in borrow.header.arguments.iter() {
      ty::verify_type(lazy, &argument.ty, tasks)?;
    };

    {
      let root = lazy.get_root_module(borrow.parent);

      print_once_per_thread!(lazy, {
        level: Stub,
        force: false,
        description: line_dbg!("verify that main arguments match expected function signature").into(),
        contents: MessageContents::File(root),
      });
    };

    // verify body
    expr::verify_block(lazy, &borrow.body, Some(&ret_ty_pair), tasks)?;

    Ok(())
  })
}
