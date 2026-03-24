use crate::lang::span::GetSpan;
use crate::lang::ty::{Intrinsic, Type};
use crate::resolve::tasks::Tasks;
use crate::resolve::coerce::{Coerce, SpecialPair};
use crate::lang::reference::{Store, TypeReference};

use super::*;

macro_rules! print_once_per_thread {
  ($lazy:ident, $x:tt) => {
    unsafe {
      static mut DID_RUN: bool = false;

      if !DID_RUN {
        DID_RUN = true;

        use crate::error::*;
        print_message($lazy, PrintableMessage $x);
      };
    };
  };
}

pub(super) fn verify_function(lazy: &Lazy, function: FunctionReference, tasks: &mut Tasks) -> Result<()> {
  // get main function
  let function_borrow = lazy.rget(function);

  // verify return type
  let ret_ty = &function_borrow.header.ret_ty;
  ty::verify_type(lazy, ret_ty)?;

  // set up some perfunctory data to coerce return type to i32
  // TODO: eventually just coerce main as fn(...) -> ...
  let ret_ty_reference = TypeReference::ReturnTypeOf(function);
  let ret_ty_pair = SpecialPair(&ret_ty_reference, ret_ty);
  {
    let ret_ty_span = ret_ty_reference.get_span(lazy);

    let undeniable_i32 = Type::Intrinsic {
      kind: Intrinsic::I32,
      span: ret_ty_span,
    };
    let undeniable_i32 = SpecialPair(&ret_ty_reference, &undeniable_i32);

    ret_ty_pair.coerce(lazy, &undeniable_i32, tasks)?;
  };

  // verify argument types
  for argument in function_borrow.header.arguments.iter() {
    ty::verify_type(lazy, &argument.ty)?;
  };

  {
    let root = lazy.get_root_module(function_borrow.parent);

    print_once_per_thread!(lazy, {
      level: Level::Debug,
      force: false,
      description: format!("stub: verify that main arguments match expected function signature"),
      contents: MessageContents::File(root),
    });
  };

  // verify body
  expr::verify_block(lazy, &function_borrow.body, &ret_ty_pair, tasks)?;

  todo!()
}
