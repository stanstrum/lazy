use crate::print_once_per_thread;

use crate::resolve::TypePair;
use crate::lang::ty::{Intrinsic, Type};
use crate::lang::reference::{FunctionReference, TypeReference};

use super::*;

pub(in crate::resolve) fn verify_function(lazy: &Lazy, function: &FunctionReference, tasks: &mut Tasks) -> Result<()> {
  // get main function
  let borrow = function.rget_from(lazy);

  let parent = lazy.describe_module(borrow.parent);
  let name = lazy.pool.get(borrow.header.name.id).collect::<String>();

  tasks.work(format!(line_dbg!("Verify function: {}::{}"), parent, name), |tasks| {
    let ret_ty_reference = TypeReference::ReturnTypeOf(*function);

    // verify return type
    let ret_ty_pair = tasks.work(line_dbg!("verify return type").into(),
    |tasks| -> Result<TypePair> {
        let ret_ty = &borrow.header.ret_ty;
        ty::verify_type(lazy, ret_ty)?;

        // set up some perfunctory data to coerce return type to i32
        // TODO: eventually just coerce main as fn(...) -> ...
        let ret_ty_pair = TypePair::new(ret_ty_reference, ret_ty.clone());
        {
          let span = dbg!(&borrow.header.ret_ty).get_span(lazy);

          ret_ty_pair.coerce(lazy, &Type::Intrinsic {
            kind: Intrinsic::I32,
            span,
          }, tasks)?;
        };

        Ok(ret_ty_pair)
      },
    )?;

    // verify argument types
    for argument in borrow.header.arguments.iter() {
      ty::verify_type(lazy, &argument.ty)?;
    };

    {
      let root = lazy.get_root_module(borrow.parent);

      print_once_per_thread!(lazy, {
        level: Level::Debug,
        force: false,
        description: line_dbg!("stub: verify that main arguments match expected function signature").into(),
        contents: MessageContents::File(root),
      });
    };

    // verify body
    expr::verify_block(lazy, &borrow.body, Some(&ret_ty_pair), tasks)?;

    Ok(())
  })
}
