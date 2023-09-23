/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use super::*;

impl Checker {
  pub fn resolve_type(&mut self, ast: &mut TypeAST) -> TypeCheckResult<()> {
    match &mut ast.e.clone() {
      Type::Intrinsic(_) | Type::Defined(_) => Ok(()),
      Type::ConstReferenceTo(ast)
      | Type::MutReferenceTo(ast)
      | Type::ConstPtrTo(ast)
      | Type::MutPtrTo(ast) => {
        self.resolve_type(ast)
      },
      Type::ArrayOf(lit, ast) => {
        let lit = lit.as_ref();

        if lit.is_some_and(|lit| !is_valid_array_length(&lit)) {
          return InvalidTypeSnafu {
            text: "Array length is invalid",
            span: lit.unwrap().span.clone()
          }.fail();
        };

        self.resolve_type(&mut **ast)
      },
      Type::Unknown(qual) => {
        let mut res_stack: Vec<_> = self.stack.iter()
          .filter_map(|scope|
            match scope {
              | ScopePointer::Function(_)
              | ScopePointer::Expression(_)
              | ScopePointer::Block(_) => None,
              ScopePointer::Namespace(ns) =>
              Some(*ns)
            }
          ).collect();

        let last_idx = qual.parts.len() - 1;
        let part_iter = qual.parts
          .iter()
          .enumerate()
          .map(
            |(i, ident)|
              (i == last_idx, ident.text.to_owned())
          );

        for (is_last, part) in part_iter {
          let map = unsafe { &mut (**res_stack.last().unwrap()).map };

          match (is_last, map.get_mut(&part)) {
            (false, Some(Structure::Namespace(ns))) => {
              res_stack.push(&mut *ns);
            },
            (true, Some(Structure::TypeAlias(alias))) => {
              self.resolve_type(&mut alias.ty)?;

              ast.e = Type::Defined(&alias.ty);
            },
            _ => {
              dbg!(is_last, map.get_mut(&part));

              return UnknownIdentSnafu {
                text: qual.to_hashable(),
                span: qual.span.clone()
              }.fail();
            }
          }
        };

        Ok(())
      },
      Type::Unresolved => todo!("type alias dest unresolved"),
    }
  }

  pub fn resolve_alias(&mut self, alias: &mut TypeAliasAST) -> TypeCheckResult<()> {
    self.resolve_type(&mut alias.ty)
  }
}
