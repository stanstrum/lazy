/* Copyright (c) 2023 Stan Strum
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

  fn resolve_qualified(&self, qual: &mut QualifiedAST) -> TypeCheckResult<VariableReference> {
    println!("{}", qual.to_string());

    let mut dup_qual = qual.clone();

    let mut res_stack = self.stack
      .iter()
      .filter_map(
        |ptr|
          match ptr {
            ScopePointer::Namespace(ns) => Some(*ns),
            _ => None
          }
      )
      .collect::<Vec<_>>();

    let last = dup_qual.parts.pop().unwrap().text;
    let parts = dup_qual.parts
      .iter()
      .map(|part| part.text.to_owned())
      .collect::<Vec<_>>();

    for part in parts {
      let map = unsafe {
        &mut (**res_stack.last().unwrap()).map
      };

      match map.get_mut(&part) {
        Some(Structure::Namespace(ns)) => {
          res_stack.push(ns);
        },
        _ if part == "super" => {
          res_stack.pop();
        },
        _ => {
          return UnknownIdentSnafu {
            text: qual.to_string(),
            span: qual.span()
          }.fail();
        },
      };
    };

    let map = unsafe {
      &mut (**res_stack.last().unwrap()).map
    };

    match map.get(&last) {
      Some(Structure::Function(func)) => {
        Ok(VariableReference::ResolvedFunction(func))
      },
      _ => InvalidTypeSnafu {
        text: qual.to_string(),
        span: qual.span()
      }.fail()
    }
  }

  pub fn resolve_variable(&self, qual: &mut QualifiedAST) -> TypeCheckResult<VariableReference> {
    let blocks = self.get_block_expr_scopes();

    if qual.parts.len() != 1 {
      return self.resolve_qualified(qual);
    };

    let name = unsafe { qual.parts.get_unchecked(0) };

    for block in blocks.iter().rev() {
      let block = unsafe { &**block };

      if block.vars.contains_key(name) {
        return Ok(VariableReference::ResolvedVariable(*block.vars.get(name).unwrap()));
      };
    };

    UnknownIdentSnafu {
      text: name.text.to_owned(),
      span: qual.span.clone()
    }.fail()
  }
}
