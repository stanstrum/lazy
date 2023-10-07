/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use inkwell::values::FunctionValue;

use crate::aster::ast::*;

use super::{
  Codegen,
  CodeGenResult
};

impl<'a, 'ctx> Codegen<'a, 'ctx> {
  pub fn generate_namespace(&mut self, ns: &NamespaceAST) -> CodeGenResult<()> {
    let mut asts_values: Vec<(&FunctionAST, FunctionValue<'ctx>)> = vec![];

    for (_, structure) in ns.map.iter() {
      match structure {
        Structure::Namespace(ns) => {
          self.generate_namespace(ns)?;
        },
        Structure::Function(func) => {
          asts_values.push((
            func,
            self.declare_function(func)?
          ));
        },
        _ => {}
      };
    };

    for (ast, value) in asts_values {
      let mut ast_params = ast.decl.args
        .values()
        .collect::<Vec<_>>();

      ast_params.sort_by_key(|ty_ast| ty_ast.span().start);

      for (param, ty) in value.get_param_iter().zip(ast_params.iter()) {
        let var_ref = VariableReference::ResolvedArgument(*ty);

        self.var_map.insert(
          var_ref,
          param
            .try_into()
            .expect("failed to convert param to ptr")
        );
      };

      self.generate_function(ast, value)?;
    };

    Ok(())
  }
}

