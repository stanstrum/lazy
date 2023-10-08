/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use inkwell::{values::{FunctionValue, AnyValue}, module::Linkage};

use crate::aster::ast::*;

use super::{
  Codegen,
  CodeGenResult
};

impl<'a, 'ctx> Codegen<'a, 'ctx> {
  pub fn generate_namespace(&mut self, ns: &NamespaceAST) -> CodeGenResult<()> {
    let mut asts_values: Vec<(&FunctionAST, FunctionValue<'ctx>)> = vec![];

    for structure in ns.map.values() {
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
        Structure::ExternDecl(decl) => {
          let name = &decl.ident.text;

          let ret_ty = self.generate_type(&decl.ret.e)?;

          let mut tys = decl.args
            .values()
            .collect::<Vec<_>>();
          tys.sort_by_key(|arg| arg.span().start);

          let arg_tys = tys.iter()
            .map(|ty| self.generate_type(&ty.e))
            .collect::<Result<Vec<_>, _>>()?
            .iter()
            .map(|meta_ty| meta_ty.to_basic_metadata())
            .collect::<Vec<_>>();

          let fn_ty = ret_ty.fn_type(arg_tys.as_slice(), decl.varargs);
          let value = self.module.add_function(name, fn_ty, Some(Linkage::External));

          self.var_map.insert(VariableReference::ResolvedExternal(decl), value.as_any_value_enum());
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
          param.as_any_value_enum()
        );
      };

      self.generate_function(ast, value)?;
    };

    Ok(())
  }
}

