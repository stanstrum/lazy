use crate::lang::function::ExprId;
use crate::lang::ty::Type;
use crate::lang::reference::{BlockReference, ExpressionReference, TypeReference};
use crate::resolve::coerce::{Coerce, SpecialPair, TypePair};

use super::*;

pub(super) fn verify_block(lazy: &Lazy, block: &BlockReference, ret_ty: &TypePair, tasks: &mut Tasks) -> Result<()> {
  let block_borrow = block.rget_from(lazy);

  let block_type_reference = TypeReference::Block(*block);
  let block_out = SpecialPair(&block_type_reference, &block_borrow.out);

  block_out.coerce(lazy, ret_ty, tasks)?;

  todo!()
}
