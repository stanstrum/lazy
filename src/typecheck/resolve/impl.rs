/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use super::*;

impl Checker {
  fn resolve_member_fn(&mut self, func: &mut MemberFunctionAST) -> TypeCheckResult<()> {
    let args = &mut func.decl.decl.args;

    for (ident, ty) in args.iter_mut() {
      // todo: make sure arg idents are unique

      self.resolve_type(ty)?;
    };

    let block = &mut func.body;

    self.stack.push(ScopePointer::Block(block));
    self.resolve_block_expression(block)?;
    self.stack.pop();

    Ok(())
  }

  pub fn resolve_impl(&mut self, _impl: &mut Impl) -> TypeCheckResult<()> {
    let ptr = _impl as *const _;

    let (methods, r#trait) = {
      match _impl {
        Impl::Impl(r#impl) => {
          self.resolve_type(&mut r#impl.ty)?;

          self.impls.push((Type::Defined(&r#impl.ty), ptr));

          (&mut r#impl.methods, None)
        }
        Impl::ImplFor(impl_for) => {
          self.resolve_type(&mut impl_for.ty)?;

          self.impls.push((Type::Defined(&impl_for.ty), ptr));

          (&mut impl_for.methods, Some(&mut impl_for.r#trait))
        }
      }
    };

    // find trait
    if r#trait.is_some() {
      todo!("locate trait and resolve");
    };

    for method in methods.iter_mut() {
      self.stack.push(ScopePointer::new_member_fn(method));
      self.resolve_member_fn(method)?;
      self.stack.pop();
    };

    Ok(())
  }
}
