/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::collections::HashMap;

use crate::typecheck::{
  Checker,
  TypeCheckResult,
  errors::*,
  extends,
  extends::assignable,
  TypeOf
};

use crate::aster::{
  ast::*,
  intrinsics
};

impl Checker {
  pub fn resolve_atom(&mut self, atom: &mut AtomExpressionAST, coerce_to: Option<&Type>) -> TypeCheckResult<Type> {
    match &mut atom.a {
      AtomExpression::Literal(lit) => {
        let span = &lit.span;

        match &lit.l {
          Literal::UnicodeString(unicode) => {
            let len = LiteralAST {
              span: span.clone(),
              l: Literal::IntLiteral(unicode.len().to_string()),
            };

            let array = Type::ArrayOf(
              Some(len),
              Box::new(TypeAST {
                span: span.clone(),
                e: Type::Intrinsic(intrinsics::U32)
              })
            );

            let array_reference = Type::ConstReferenceTo(Box::new(TypeAST {
              span: span.clone(),
              e: array,
            }));

            let len = Type::Intrinsic(intrinsics::U64);

            let ident = IdentAST {
              span: lit.span(),
              text: "str".to_string(),
            };

            let ptr_ident = IdentAST {
              span: lit.span(),
              text: "ptr".to_string(),
            };

            let len_ident = IdentAST {
              span: lit.span(),
              text: "len".to_string(),
            };

            let slice_struct = Type::Struct((&ident).into(), vec![
              (array_reference, ptr_ident),
              (len, len_ident),
            ]);

            atom.out = slice_struct;
          },
          Literal::ByteString(text) => {
            let len = LiteralAST {
              span: span.clone(),
              l: Literal::IntLiteral(text.len().to_string())
            };

            let array = Type::ArrayOf(
              Some(len),
              Box::new(TypeAST {
                span: span.clone(),
                e: Type::Intrinsic(intrinsics::U8)
              })
            );

            let array_reference = Type::ConstReferenceTo(
              Box::new(TypeAST {
                span: span.clone(),
                e: array,
              })
            );

            atom.out = array_reference;
          },
          Literal::CString(text) => {
            // include extra byte for null-terminator
            let size = text.len() + 1;

            let len = LiteralAST {
              span: span.clone(),
              l: Literal::IntLiteral(size.to_string())
            };

            let array = Type::ArrayOf(
              Some(len),
              Box::new(TypeAST {
                span: span.clone(),
                e: Type::Intrinsic(intrinsics::U8)
              })
            );

            let array_reference = Type::ConstReferenceTo(
              Box::new(TypeAST {
                span: span.clone(),
                e: array,
              })
            );

            atom.out = array_reference;
          },
          Literal::Char(_) => todo!("resolve char"),
          Literal::ByteChar(_) => todo!("resolve bytechar"),
          Literal::FloatLiteral(_) => todo!("resolve float literal"),
          Literal::IntLiteral(_) => {
            let Some(coerce_to) = coerce_to else {
              todo!("error: int literal has no type coercion");
            };

            if extends(coerce_to, &Type::Intrinsic(intrinsics::U8)) {
              atom.out = coerce_to.clone();
            } else {
              return IncompatibleTypeSnafu {
                span: atom.span(),
                what: "Integer literal",
                with: coerce_to.to_string()
              }.fail();
            };
          },
        };
      },
      AtomExpression::UnresolvedVariable(qual) => {
        let resolved = self.resolve_variable(qual)?;
        let out = resolved.type_of();

        atom.a = AtomExpression::ValueVariable(qual.to_owned(), resolved);

        if let Some(out) = out {
          atom.out = out;
        } else {
          panic!("failed to resolve atom type `{}`", atom.to_string());
        };
      },
      AtomExpression::StructInitializer(initializer) => {
        let mut initializer_ty = self.resolve_fqual_to_type(&initializer.fqual)?;

        atom.out = initializer_ty.clone();

        // "if you want something done, do it yourself"
        while let Type::Defined(ty_ast) = initializer_ty {
          let ty_ast = unsafe { &*ty_ast };

          initializer_ty = ty_ast.e.to_owned();
        };

        let Type::Struct(_fqual, members) = initializer_ty else {
          return InvalidTypeSnafu {
            text: "Initializer is not a struct",
            span: initializer.fqual.span(),
          }.fail();
        };

        // let r#struct = unsafe { &*r#struct };

        let init_len = initializer.members.len();
        let struct_len = members.len();
        if init_len != struct_len {
          return IncompatibleTypeSnafu {
            span: initializer.span(),
            what: "Struct initializer fields",
            with: format!("struct definition (expected {struct_len}, got {init_len})"),
          }.fail();
        };

        let mut item_map = HashMap::<IdentAST, Type>::new();

        for (ty, ident) in members.iter() {
          item_map.insert(ident.to_owned(), ty.to_owned());
        };

        for (ident, expr) in initializer.members.iter_mut() {
          let Some(field_ty) = item_map.remove(ident) else {
            todo!("bad field");
          };

          let expr_ty = self.resolve_expression(expr, Some(&field_ty))?;

          if !assignable(&expr_ty, &field_ty) {
            todo!("types dont match")
          };
        };
      },
      _ => todo!("{:#?}", &atom.a),
    };

    Ok(atom.out.clone())
  }
}
