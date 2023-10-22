/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use super::*;

impl Checker {
  pub fn follow_structure<'a>(structure: &'a Structure) -> &'a Structure {
    match structure {
      Structure::ImportedStructure { structure, .. } => {
        let structure = unsafe { &**structure };

        Self::follow_structure(structure)
      },
      _ => structure
    }
  }

  pub fn follow_structure_mut<'a>(structure: &'a mut Structure) -> &'a mut Structure {
    match structure {
      Structure::ImportedStructure { structure, .. } => {
        let structure = unsafe { &mut **structure };

        Self::follow_structure_mut(structure)
      },
      _ => structure
    }
  }

  fn get_block_expr_scopes(&self) -> Vec<*mut BlockExpressionAST> {
    self.stack.iter().filter_map(|ptr|
      match ptr {
        ScopePointer::Block(block) => Some(*block),
        _ => None
      }
    ).collect()
  }

  fn resolve_qualified(&self, qual: &QualifiedAST) -> TypeCheckResult<VariableReference> {
    // println!("{}", qual.to_string());

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

      match map.get_mut(&part).map(Self::follow_structure_mut) {
        Some(Structure::Namespace(ns)) => {
          res_stack.push(ns);
        },
        Some(Structure::ImportedNamespace { ns, .. }) => {
          let ns = unsafe { &mut **ns };

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

    match map.get(&last).map(Self::follow_structure) {
      Some(Structure::Function(func)) => {
        Ok(VariableReference::ResolvedFunction(func))
      },
      Some(Structure::ExternDecl(decl)) => {
        Ok(VariableReference::ResolvedExternal(decl))
      },
      Some(other) => {
        InvalidTypeSnafu {
          text: format!("Cannot use (is {:#?}) as a variable: {}", other, qual.to_string()),
          span: qual.span()
        }.fail()
      },
      None => {
        UnknownIdentSnafu {
          text: qual.to_string(),
          span: qual.span(),
        }.fail()
      }
    }
  }

  fn resolve_arg_var(&self, qual: &QualifiedAST) -> Option<VariableReference> {
    if qual.parts.len() != 1 {
      return None;
    };

    let ident = qual.parts.last().unwrap();

    let decl = self.stack.iter().find_map(
      |ptr|
        match ptr {
          ScopePointer::Function(func) => {
            Some(unsafe {
              &(**func).decl
            })
          },
          ScopePointer::MemberFunction(member_func) => {
            Some(unsafe {
              &(**member_func).decl.decl
            })
          },
          _ => None
        }
      ).unwrap();

    if decl.args.contains_key(ident) {
      let arg = decl.args.get(ident).unwrap();

      Some(VariableReference::ResolvedArgument(arg))
    } else {
      None
    }
  }

  fn resolve_local_variable(&self, qual: &QualifiedAST) -> Option<VariableReference> {
    if qual.parts.len() != 1 {
      return None;
    };

    let blocks = self.get_block_expr_scopes();

    let name = unsafe { qual.parts.get_unchecked(0) };

    for block in blocks.iter().rev() {
      let block = unsafe { &**block };

      if block.vars.contains_key(name) {
        let binding = *block.vars.get(name).unwrap();

        return Some(VariableReference::ResolvedVariable(binding));
      };
    };

    None
  }

  pub fn resolve_variable(&self, qual: &QualifiedAST) -> TypeCheckResult<VariableReference> {
    if let Some(local_var) = self.resolve_local_variable(qual) {
      Ok(local_var)
    } else if let Some(arg_var) = self.resolve_arg_var(qual) {
      Ok(arg_var)
    } else {
      self.resolve_qualified(qual)
    }
  }
}
