/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::HashMap;

use super::*;

impl TemplateAST {
  pub fn to_map(&self) -> HashMap<IdentAST, Type> {
    let mut map = HashMap::<IdentAST, Type>::new();

    for constraint in self.constraints.iter() {
      match constraint {
        TemplateConstraint::Unconstrained(ident) => {
          map.insert(
            ident.clone(),
            Type::Generic(
              ident.clone(), vec![]
            )
          );
        },
        TemplateConstraint::Extends(_, _) => todo!(),
      }
    };

    map
  }
}

impl Checker {
  pub fn get_generics(&self) -> HashMap<IdentAST, Type> {
    let template_iter = self.stack.iter()
      .filter_map(|ptr| {
        let ScopePointer::Template(template) = ptr else {
          return None
        };

        Some(unsafe { &**template })
      });

    let mut map = HashMap::<IdentAST, Type>::new();
    for template in template_iter {
      for (k, v) in template.to_map() {
        map.insert(k, v);
      };
    };

    map
  }

  pub fn resolve_type(&mut self, ast: &mut TypeAST) -> TypeCheckResult<()> {
    let generics = self.get_generics();

    let what_to_replace_with = 'replace_with: {
      match &mut ast.e {
        Type::Intrinsic(_)
        | Type::Defined(_)
        | Type::Struct(_)
        | Type::Function(_)
        | Type::External(_) => {
          return Ok(());
        },
        Type::ConstReferenceTo(ast)
        | Type::MutReferenceTo(ast)
        | Type::ConstPtrTo(ast)
        | Type::MutPtrTo(ast) => {
          self.resolve_type(ast)?;

          return Ok(());
        },
        Type::ArrayOf(lit, ast) => {
          let lit = lit.as_ref();

          if lit.is_some_and(|lit| !is_valid_array_length(lit)) {
            return InvalidTypeSnafu {
              text: "Array length is invalid",
              span: lit.unwrap().span()
            }.fail();
          };

          self.resolve_type(ast)?;

          return Ok(());
        },
        Type::Unknown(fqual) => {
          'find_generic: {
            if fqual.parts.len() != 1 {
              break 'find_generic;
            };

            let fident = fqual.parts.first().unwrap();

            let Some(generic_ty) = generics.get(&fident.ident) else {
              break 'find_generic
            };

            if fident.generics.is_some() {
              todo!("error for generics on a generic");
            };

            break 'replace_with generic_ty.to_owned();
          };

          let mut res_stack: Vec<_> = self.stack.iter()
            .filter_map(|scope|
              match scope {
                ScopePointer::Namespace(ns) => Some(*ns),
                _ => None
              }
            ).collect();

          let last_idx = fqual.parts.len() - 1;
          let part_iter = fqual.parts
            .iter()
            .enumerate()
            .map(
              |(i, ident)|
                (i == last_idx, ident)
            );

          for (is_last, part) in part_iter {
            let map = unsafe { &mut (**res_stack.last().unwrap()).map };

            match (is_last, map.get_mut(&part.to_hashable()).map(Self::follow_structure_mut)) {
              (false, Some(Structure::Namespace(ns))) => {
                res_stack.push(ns);
              },
              (false, Some(Structure::ImportedNamespace { ns, .. })) => {
                let ns = unsafe { &mut **ns };

                res_stack.push(ns);
              },
              (false, _) if part.ident.text == "super" => {
                res_stack.pop();
              },
              (true, Some(Structure::TypeAlias(alias))) => {
                self.resolve_type(&mut alias.ty)?;

                break 'replace_with Type::Defined(&alias.ty);
              },
              (true, Some(Structure::Function(func))) => {
                break 'replace_with Type::Function(func);
              },
              (true, Some(Structure::Struct(r#struct))) => {
                break 'replace_with Type::Struct(r#struct);
              },
              _ => {
                return UnknownIdentSnafu {
                  text: fqual.to_hashable(),
                  span: fqual.span()
                }.fail();
              }
            }
          };

          return Ok(());
        },
        Type::Generic(..) => todo!("resolve_type for generic"),
        Type::UnresolvedLiteral(_)
        | Type::Unresolved => todo!("type alias dest unresolved"),
      };
    };

    ast.e = what_to_replace_with;

    Ok(())
  }
}
