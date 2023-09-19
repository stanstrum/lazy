/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use super::*;

impl Checker {
  fn get_block_expr_scopes(&self) -> Vec<*const BlockExpressionAST> {
    self.stack.iter().filter_map(|ptr|
      match ptr {
        | ScopePointer::Namespace(_)
        | ScopePointer::Function(_)
        | ScopePointer::Block(_) => None,
        ScopePointer::Expression(expr) => {
          if let Expression::Block(block) = unsafe { &mut **expr } {
            Some(block as *const _)
          } else {
            None
          }
        },
      }
    ).collect()
  }

  pub fn resolve_variable(&self, qual: &mut QualifiedAST) -> TypeCheckResult<*const BindingAST> {
    let blocks = self.get_block_expr_scopes();

    if qual.parts.len() != 1 {
      println!("{}", qual.to_string());

      return NotImplementedSnafu {
        what: "Qualified variable"
      }.fail();
    };

    let name = unsafe { qual.parts.get_unchecked(0) };

    for block in blocks.iter().rev() {
      let block = unsafe { &**block };

      if block.vars.contains_key(name) {
        return Ok(*block.vars.get(name).unwrap());
      };
    };

    UnknownIdentSnafu {
      text: name.text.to_owned(),
      span: qual.span.clone()
    }.fail()
  }
}
