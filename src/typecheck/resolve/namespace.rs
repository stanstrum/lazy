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
    let mut names = map
      .keys()
      .map(|name| name.to_owned())
      .collect::<Vec<String>>();

    let get_start = |name: &String|
      map.get(name).unwrap().span().start;

    names.sort_by(|a, b| {
      let a = get_start(a);
      let b = get_start(b);

      a.cmp(&b)
    });

    for name in names.iter() {
      match map.get_mut(name).unwrap() {
        Structure::TypeAlias(alias) => {
          self.resolve_alias(alias)?;
        },
        Structure::Namespace(ns) => {
          self.stack.push(ScopePointer::Namespace(ns));
          self.resolve_ns(ns)?;
          self.stack.pop();
        },
        Structure::Function(func) => {
          self.stack.push(ScopePointer::Function(func));
          self.resolve_function(func)?;
          self.stack.pop();
        },
        Structure::Struct(r#struct) => {
          for (ty, _) in r#struct.members.iter_mut() {
            self.resolve_type(ty)?;
          };
        },
        Structure::Trait(_) => todo!("resolve trait"),
        Structure::Impl(r#impl) => {
          self.stack.push(ScopePointer::Impl(r#impl));
          self.resolve_impl(r#impl)?;
          self.stack.pop();
        },
        Structure::ExternDecl(decl) => {
          self.resolve_type(&mut decl.ret)?;

          for ty in decl.args.values_mut() {
            self.resolve_type(ty)?;
          };
        },
        Structure::Import(_) => todo!("resolve_ns import"),
        Structure::Imported(_) => {},
      };
    };

    Ok(())
  }
}
