/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use super::*;

impl Checker {
  pub fn resolve_ns(&mut self, ns: &mut NamespaceAST) -> TypeCheckResult<()> {
    let map = &mut ns.map;

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
          self.resolve_ns(ns)?;
          self.stack.pop();
        },
        Structure::Function(func) => {
          self.stack.push(ScopePointer::Function(func));
          self.resolve_function(func)?;
          self.stack.pop();
        },
        Structure::Struct(r#struct) => {
          for (ty, ident) in r#struct.members.iter_mut() {
            self.resolve_type(ty)?;
          };

          for scope in self.stack.iter().rev() {
            match scope {
              ScopePointer::Namespace(_) => todo!(),
              ScopePointer::Function(_) => todo!(),
              ScopePointer::Block(_) => todo!(),
              ScopePointer::Expression(_) => todo!(),
            }
          };
        },
        Structure::Trait(_) => todo!(),
        Structure::Impl(_) => todo!(),
      };
    };

    Ok(())
  }
}
