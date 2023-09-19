/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use super::{
  Checker,
  ScopePointer,
  errors::*
};

use crate::aster::ast::*;

trait IsResolved {
  fn is_resolved(&self) -> bool;
}

impl IsResolved for TypeAST {
  fn is_resolved(&self) -> bool {
    self.e.is_resolved()
  }
}

impl IsResolved for Type {
  fn is_resolved(&self) -> bool {
    match self {
      Type::Intrinsic(_) => true,
      | Type::ConstReferenceTo(ast)
      | Type::MutReferenceTo(ast)
      | Type::ConstPtrTo(ast)
      | Type::MutPtrTo(ast) => ast.is_resolved(),
      Type::ArrayOf(_, _) => todo!("is_resolved arrayof"),
      Type::Defined(ast) => unsafe { (**ast).is_resolved() },
      Type::Unknown(_) => false,
      Type::Unresolved => false,
    }
  }
}

impl Checker {
  pub fn new(global: &mut NamespaceAST) -> Self {
    Self {
      stack: vec![
        ScopePointer::new_ns(global)
      ]
    }
  }

  fn get_scope(&self) -> ScopePointer {
    *self.stack
      .last()
      .unwrap()
  }

  fn resolve_type(&mut self, ast: &mut TypeAST) -> TypeCheckResult<()> {
    match unsafe { &mut ast.e } {
      Type::Intrinsic(_) | Type::Defined(_) => Ok(()),
      Type::ConstReferenceTo(ast)
      | Type::MutReferenceTo(ast)
      | Type::ConstPtrTo(ast)
      | Type::MutPtrTo(ast) => {
        self.resolve_type(ast)
      },
      Type::ArrayOf(lit, ast) => {
        todo!("resolve array of");
      },
      Type::Unknown(qual) => {
        let mut res_stack: Vec<_> = self.stack.iter()
          .filter_map(|scope|
            match scope {
              ScopePointer::Function(_) | ScopePointer::Expression(_) => None,
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
            },
            _ => {
              return UnknownIdentSnafu {
                text: qual.to_hashable()
              }.fail();
            }
          }
        };

        Ok(())
      },
      Type::Unresolved => todo!("type alias dest unresolved"),
    }
  }

  fn resolve_alias(&mut self, alias: &mut TypeAliasAST) -> TypeCheckResult<()> {
    self.resolve_type(&mut alias.ty)?;

    Ok(())
  }

  fn resolve_function(&mut self) -> TypeCheckResult<()> {
    todo!("resolve_function");

    self.stack.pop();

    Ok(())
  }

  pub fn resolve_ns(&mut self) -> TypeCheckResult<()> {
    let ScopePointer::Namespace(ns) = self.get_scope() else {
      unreachable!();
    };

    let map = unsafe { &mut (*ns).map };

    // resolve names
    let names = map
      .keys()
      .map(|name| name.to_owned())
      .collect::<Vec<String>>();

    for name in names.iter() {
      match map.get_mut(name).unwrap() {
        Structure::TypeAlias(alias) => {
          self.resolve_alias(alias)?;
        },
        Structure::Namespace(ns) => {
          self.stack.push(ScopePointer::new_ns(ns));
          self.resolve_ns()?;
        },
        Structure::Function(func) => {
          self.stack.push(ScopePointer::Function(func));
          self.resolve_function()?;
        },
        Structure::Struct(r#struct) => {
          for (ty, ident) in r#struct.members.iter_mut() {
            self.resolve_type(ty)?;
          };

          for scope in self.stack.iter().rev() {
            match scope {
              ScopePointer::Namespace(_) => todo!(),
              ScopePointer::Function(_) => todo!(),
              ScopePointer::Expression(_) => todo!(),
            }
          };
        },
        Structure::Trait(_) => todo!(),
        Structure::Impl(_) => todo!(),
      };
    };

    self.stack.pop();

    Ok(())
  }
}
