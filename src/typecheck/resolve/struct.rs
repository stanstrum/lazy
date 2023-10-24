/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::typecheck::{
  Checker,
  TypeCheckResult,
  ScopePointer
};

use crate::aster::ast::*;

impl Checker {
  pub fn get_struct_member_idx(r#struct: &StructAST, ident: &IdentAST) -> TypeCheckResult<(Type, usize)> {
    for (i, (memb_ty, member_ident)) in r#struct.members.iter().enumerate() {
      if ident == member_ident {
        return Ok((Type::Defined(memb_ty), i));
      };
    };

    todo!("error for ident not found");
  }

  pub fn resolve_struct(&mut self, r#struct: &mut StructAST) -> TypeCheckResult<()> {
    if let Some(template) = &mut r#struct.template {
      self.stack.push(ScopePointer::Template(template));
    };

    for (ty, _) in r#struct.members.iter_mut() {
      self.resolve_type(ty)?;
    };

    if r#struct.template.is_some() {
      self.stack.pop();
    };

    Ok(())
  }
}
