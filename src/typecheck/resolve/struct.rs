/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::typecheck::{
  Checker,
  TypeCheckResult
};

use crate::aster::ast::*;

impl Checker {
  // fn get_qualified_structure<'a>(&'a self, qual: &QualifiedAST) -> TypeCheckResult<&'a Structure> {
  //   let mut stack = self.stack.iter().filter_map(|ptr| match ptr {
  //     ScopePointer::Namespace(ns) => {
  //       Some(unsafe { &**ns })
  //     },
  //     _ => None
  //   }).collect::<Vec<_>>();

  //   let (last, scopes) = qual.parts.split_last().unwrap();

  //   for part in scopes {
  //     let map = &stack.last().unwrap().map;

  //     match map.get(&part.to_hashable()) {
  //       Some(Structure::Namespace(ns)) => {
  //         stack.push(ns);
  //       },
  //       Some(Structure::ImportedNamespace { ns, .. }) => {
  //         let ns = unsafe { &**ns };

  //         stack.push(ns);
  //       },
  //       _ if part.text == "super" => {
  //         stack.pop();
  //       },
  //       Some(_) => {
  //         return InvalidTypeSnafu {
  //           text: format!("{} is not a namespace", &part.text),
  //           span: part.span(),
  //         }.fail();
  //       },
  //       None => {
  //         return UnknownIdentSnafu {
  //           text: &part.text,
  //           span: part.span(),
  //         }.fail();
  //       }
  //     };
  //   };

  //   let last_ns = stack.last().unwrap();

  //   match last_ns.map.get(&last.to_hashable()) {
  //     Some(structure) => Ok(Self::follow_structure(structure)),
  //     None => UnknownIdentSnafu {
  //       text: &last.text,
  //       span: last.span(),
  //     }.fail()
  //   }
  // }

  // fn get_impls_for(&self, ty: &Type) -> TypeCheckResult<HashMap<IdentAST, VariableReference>> {
  //   let mut map = HashMap::<IdentAST, *const MemberFunctionAST>::new();

  //   for (implemented_ty, r#impl) in self.impls.iter() {
  //     let r#impl = unsafe { &**r#impl };

  //     let methods = match r#impl {
  //       Impl::Impl(r#impl) => {
  //         &r#impl.methods
  //       },
  //       Impl::ImplFor(impl_for) => {
  //         &impl_for.methods
  //       }
  //     };

  //     if extends(ty, implemented_ty) {
  //       for method in methods.iter() {
  //         let ident = &method.decl.decl.ident;

  //         if map.contains_key(ident) {
  //           let original = unsafe { &**map.get(ident).unwrap() };

  //           let span_a = original.span();
  //           let span_b = method.span();

  //           return DuplicateIdentSnafu {
  //             text: ident.text.to_owned(),
  //             a: span_a,
  //             b: span_b,
  //           }.fail()
  //         };

  //         map.insert(ident.clone(), method);
  //       };
  //     };
  //   };

  //   let map = map.iter().map(
  //     |(k, v)|
  //       (k.to_owned(), VariableReference::ResolvedMemberFunction(unsafe { &**v }))
  //     ).collect::<HashMap<_, _>>();

  //   Ok(map)
  // }

  pub fn get_struct_member_idx(r#struct: &StructAST, ident: &IdentAST) -> TypeCheckResult<(Type, usize)> {
    for (i, (memb_ty, member_ident)) in r#struct.members.iter().enumerate() {
      if ident == member_ident {
        return Ok((Type::Defined(memb_ty), i));
      };
    };

    todo!("error for ident not found");
  }
}
