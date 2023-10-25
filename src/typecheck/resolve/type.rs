/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::HashMap;

use super::*;

fn typeof_struct(r#struct: &StructAST) -> Type {
  let unwrapped_members = r#struct.members.iter()
    .map(|(ast, ident)| (ast.e.to_owned(), ident.to_owned()))
    .collect::<Vec<_>>();

  Type::Struct((&r#struct.ident).into(), unwrapped_members)
}

impl TemplateAST {
  pub fn to_positional_tuple(&self) -> Vec<(IdentAST, Type)> {
    let mut tuples = Vec::<(IdentAST, Type)>::new();

    for constraint in self.constraints.iter() {
      match constraint {
        TemplateConstraint::Unconstrained(ident) => {
          tuples.push((
            ident.clone(),
            Type::Generic(
              ident.clone(), vec![]
            ))
          );
        },
        TemplateConstraint::Extends(_, _) => todo!(),
      }
    };

    tuples
  }

  pub fn to_map(&self) -> HashMap<IdentAST, Type> {
    self.to_positional_tuple().into_iter().collect()
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
        | Type::Struct(..)
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

            match (is_last, map.get_mut(&part.ident.to_hashable()).map(Self::follow_structure_mut)) {
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
                break 'replace_with typeof_struct(r#struct);
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

  fn replace_generics(ty: &mut Type, map: &HashMap<IdentAST, Type>) {
    match ty {
      Type::Generic(ident, ..) => {
        let Some(replace_ty) = map.get(&ident) else {
          todo!("error for unknown generic {ident:?}")
        };

        *ty = replace_ty.to_owned();
      },
      Type::Struct(_fqual, members) => {
        // let r#struct = unsafe { &*r#struct };

        for (memb_ty, _memb_ident) in members.iter_mut() {
          Self::replace_generics(memb_ty, map);
        };
      },
      Type::ConstReferenceTo(ty)
      | Type::MutReferenceTo(ty)
      | Type::ConstPtrTo(ty)
      | Type::MutPtrTo(ty)
      | Type::ArrayOf(_, ty) => {
        Self::replace_generics(&mut ty.e, map);
      },
      Type::Intrinsic(_) => {},
      _ => todo!("replace_generics {ty:?}")
    }
  }

  pub fn resolve_fqual_to_type(&self, fqual: &FullyQualifiedAST) -> TypeCheckResult<Type> {
    let mut stack = self.stack.iter()
      .filter_map(|ptr| {
        let ScopePointer::Namespace(ns) = ptr else {
          return None;
        };

        Some(unsafe { &**ns })
      })
      .collect::<Vec<_>>();

    let (last, parts) = fqual.parts.split_last().unwrap();

    for part in parts {
      let map = &stack.last().unwrap().map;

      let text = part.ident.to_hashable();
      match map.get(&text) {
        Some(Structure::Namespace(ns)) => {
          if part.generics.is_some() {
            todo!("error for namespace traversal with generics");
          };

          stack.push(ns);
        },
        _ if text == "super" => {
          stack.pop().unwrap();
        },
        Some(_) => todo!("unknown structure traversal"),
        None => {
          return UnknownIdentSnafu {
            span: part.span(),
            text
          }.fail();
        },
      };
    };

    let map = &stack.last().unwrap().map;
    let child = map.get(&last.ident.to_hashable());

    match child.map(Self::follow_structure) {
      Some(Structure::Struct(r#struct)) => {
        let ty = typeof_struct(r#struct);

        if let Some(template) = &r#struct.template {
          let Some(specified_generics) = &last.generics else {
            todo!("error for not satisfying generics")
          };

          let dest_generics = template.to_positional_tuple();

          let mut replace_map = HashMap::<IdentAST, Type>::new();

          if specified_generics.len() != dest_generics.len() {
            todo!("error for generic/template length mismatch");
          };

          for (specified_ty, (generic_ident, generic_ty)) in specified_generics.iter().zip(dest_generics.iter()) {
            if !extends(&specified_ty.e, generic_ty) {
              todo!("specified type does not extend generic");
            };

            replace_map.insert(generic_ident.to_owned(), Type::Defined(specified_ty));
          };

          let Type::Struct(_, unwrapped_members) = ty else {
            unreachable!();
          };

          let mut new_ty = Type::Struct(fqual.to_owned(), unwrapped_members);
          Self::replace_generics(&mut new_ty, &replace_map);

          Ok(new_ty)
        } else {
          Ok(ty)
        }
      },
      Some(_) => todo!("error: invalid initializer"),
      None => todo!(),
    }
  }
}
