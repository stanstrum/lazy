use lang::reference::{TypePartReference, TypeReference, VariableReference};
use lang::ty::{QualifiedSearchSpace, TypeValue};

use crate::function::print_function_reference;

use super::*;

impl<C: Compiler> Pretty<C> for lang::ty::Type<C> {
  type Out = String;

  fn print<'local, 'store, 'pool>(&'local self, store: &'store C::Store<'pool>) -> Self::Out {
    let reference = self.reference.print(store);

    match &self.ty {
      Some(ty) => format!("/* {reference} */ {}", ty.print(store)),
      None => format!("{reference}"),
    }
  }
}

impl<C: Compiler> Pretty<C> for TypePartReference<C> {
  type Out = String;

  fn print<'local, 'store, 'pool>(&self, store: &'store C::Store<'pool>) -> Self::Out {
    self.rget_from(store).print(store)
  }
}

impl<C: Compiler> Pretty<C> for lang::ty::Qualified<C> {
  type Out = String;

  fn print<'local, 'store, 'pool>(&self, store: &'store C::Store<'pool>) -> Self::Out {
    let mut out = String::new();

    match &self.implicit {
      QualifiedSearchSpace::Implicit => {},
      &QualifiedSearchSpace::Module(module_reference) => {
        out += &store.describe_module(module_reference);
      },
      QualifiedSearchSpace::Intrinsic { kind, .. } => {
        out += &kind.to_string();
      }
      QualifiedSearchSpace::Type(type_reference) => {
        out += &type_reference.print(store);
      },
      QualifiedSearchSpace::Struct(_) => todo!(),
    };

    out += "::";

    for (i, part) in self.parts.iter().enumerate() {
      if i != 0 {
        out += "::";
      };

      out += store.pool().get(part.id).as_str();
    };

    out
  }
}

impl<C: Compiler> Pretty<C> for TypeReference<C> {
  type Out = String;

  fn print<'local, 'store, 'pool>(&self, store: &'store C::Store<'pool>) -> Self::Out {
    match self {
      TypeReference::Part(part) => part.rget_from(store).print(store),
      TypeReference::ReturnTypeOf(function) => {
        format!("ReturnType<{}>", print_function_reference::<C>(store, function))
      },
      TypeReference::Alias(alias ) => {
        let path = store.describe_module(alias.0);
        let name = alias.rget_from(store).name.print(store);
        format!("{path}::{name}")
      },
      TypeReference::Variable(VariableReference::Argument(parent, index)) => {
        format!("ArgumentOf<{}>[{index}]", print_function_reference::<C>(store, parent))
      },
      TypeReference::Variable(v @ VariableReference::Block(block, _)) => {
        let name = v.rget_from(store).name;

        format!("typeof {{{} {}}}::{}", print_function_reference::<C>(store, &v.parent()), block.print(store), name.print(store))
      },
      TypeReference::Expression(expression) => {
        format!("typeof {{{}}}", expression.print(store))
      },
      TypeReference::Block(block) => format!("typeof {{{}}}", block.print(store)),
      TypeReference::StructMember(struct_reference, id) => {
        let struct_borrow = struct_reference.rget_from(store);
        let member = struct_borrow.members.get(*id).unwrap();
        let member_name = member.name.print(store);
        let parent_name = store.describe_module(struct_reference.0);

        format!("{parent_name}::{member_name}")
      },
    }
  }
}

impl<C: Compiler> Pretty<C> for TypeValue<C> {
  type Out = String;

  fn print<'local, 'store, 'pool>(&self, store: &'store C::Store<'pool>) -> Self::Out {
    match self {
      // Type::Reference(reference) => reference.print(store),
      TypeValue::Unresolved { qualified, .. } => format!("{{unknown}} {}", qualified.print(store)),
      TypeValue::Intrinsic { kind, .. } => kind.to_string(),
      // Type::Resolved { original, reference } => {
      //   format!("/* {deferred} */ {original}",
      //     deferred = reference.print(store),
      //     original = original.print(store),
      //   )
      // },
      TypeValue::WeakFloat { .. } => "{weak float}".into(),
      TypeValue::WeakInteger { .. } => "{weak integer}".into(),
      TypeValue::WeakString { .. } => "{weak string}".into(),
      TypeValue::ReferenceTo { ty, r#mut, .. } => format!("&{mutable}{ty}",
        mutable = if *r#mut { "mut " } else { "" },
        ty = ty.print(store),
      ),
      TypeValue::SizedArrayOf { ty, size, .. } => format!("[{size}]{}", ty.print(store)),
      TypeValue::UnsizedArrayOf { ty, .. } => format!("[]{}", ty.print(store)),
      // Type::Expression(expression) => {
      //   let fname = store[expression.function].header.name.print(store);
      //   let index = expression.index;
      //   format!("/* typeof {fname}:{index:?} */")
      // },
      TypeValue::Resolved { part, .. } => format!("|{}|", part.print(store)),
      TypeValue::Reference(reference) => format!("|{}|", reference.print(store)),
      TypeValue::Weak { .. } => "{weak}".into(),
      TypeValue::Struct { prototype } => {
        let parent_name = store.describe_module(prototype.0);
        let name = prototype.rget_from(store).name.print(store);

        format!("{{struct}} {parent_name}::{name}")
      },
      // other => todo!("{other:?}"),
    }
  }
}

impl<C: Compiler> Pretty<C> for QualifiedSearchSpace<C> {
  type Out = String;

  fn print<'local, 'store, 'pool>(&'local self, store: &'store C::Store<'pool>) -> Self::Out {
    match self {
      QualifiedSearchSpace::Implicit => "::".into(),
      QualifiedSearchSpace::Type(overwrite_type_reference) => overwrite_type_reference.print(store),
      QualifiedSearchSpace::Intrinsic { kind, .. } => kind.to_string(),
      QualifiedSearchSpace::Module(module_reference) => store.describe_module(*module_reference),
      QualifiedSearchSpace::Struct(_) => todo!(),
    }
  }
}
