/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use super::*;

impl Checker {
  fn get_block_expr_scopes(&self) -> Vec<*mut BlockExpressionAST> {
    self.stack.iter().filter_map(|ptr|
      match ptr {
        | ScopePointer::Namespace(_)
        | ScopePointer::Function(_)
        | ScopePointer::Impl(_)
        | ScopePointer::MemberFunction(_)
        | ScopePointer::Expression(_) => None,
        ScopePointer::Block(block) => Some(*block)
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
      Some(Structure::ExternDecl(decl)) => {
        Ok(VariableReference::ResolvedExternal(decl))
      }
      _ => InvalidTypeSnafu {
        text: qual.to_string(),
        span: qual.span()
      }.fail()
    }
  }

  fn resolve_arg_var(&self, ident: &IdentAST) -> Option<VariableReference> {
    let decl = self.stack.iter().find(
      |ptr|
        matches!(ptr, ScopePointer::Function(_) | ScopePointer::MemberFunction(_))
    ).map(
      |ptr|
        match ptr {
          ScopePointer::Namespace(_)
          | ScopePointer::Block(_)
          | ScopePointer::Expression(_)
          | ScopePointer::Impl(_) => unreachable!(),
          ScopePointer::Function(func) => unsafe {
            &(**func).decl
          },
          ScopePointer::MemberFunction(func) => unsafe {
            &(**func).decl.decl
          }
        }
    ).unwrap();

    if decl.args.contains_key(ident) {
      let arg = decl.args.get(ident).unwrap();

      Some(VariableReference::ResolvedArgument(arg))
    } else {
      None
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
        let binding = *block.vars.get(name).unwrap();

        return Ok(VariableReference::ResolvedVariable(binding));
      };
    };

    if let Some(arg_var) = self.resolve_arg_var(name) {
      return Ok(arg_var);
    };

    UnknownIdentSnafu {
      text: name.text.to_owned(),
      span: qual.span()
    }.fail()
  }
}
