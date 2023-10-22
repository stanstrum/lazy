/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::typecheck::{
  Checker,
  TypeCheckResult,
  TypeOf
};

use crate::aster::ast::*;

impl Checker {
  pub fn resolve_dest_atom(&mut self, atom: &mut AtomExpressionAST) -> TypeCheckResult<Type> {
    let span = atom.span();

    match &mut atom.a {
      AtomExpression::Literal(_) => todo!("resolve_dest_atom literal"),
      AtomExpression::UnresolvedVariable(qual) => {
        let var_ref = self.resolve_variable(qual)?;
        let ty = var_ref.type_of_expect(span)?;

        atom.a = AtomExpression::DestinationVariable(qual.to_owned(), var_ref);
        atom.out = ty.clone();

        Ok(ty)
      },
      AtomExpression::ValueVariable(..) => todo!("resolve_dest_atom valuevariable"),
      AtomExpression::DestinationVariable(..) => todo!("resolve_dest_atom destinationvariable"),
      AtomExpression::Return(_) => todo!("resolve_dest_atom return"),
      AtomExpression::Break(_) => todo!("resolve_dest_atom break"),
      other => todo!("resolve_dest_atom {other:?}")
    }
  }
}
