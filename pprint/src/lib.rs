use lang::token::{NumericValue, StringKind};
use lang::span::{Span, GetSpan};
use lang::reference::{BlockReference, ExpressionReference, Reference, Store, TypePartReference, TypeReference, VariableReference};
use lang::ty::{Qualified, QualifiedSearchSpace, TypeValue, Type};
use lang::function::Function;
use lang::module::{Module, Name};
use lang::token::{UnaryOperator, UnaryPrefixOperator, UnarySuffixOperator};
use lang::expr::{BlockExpression, Expression, LiteralKind};
use lang::{Compiler, CompilerPoolStore};

pub trait Pretty<C: Compiler> {
  type Out;

  fn print<'local, 'store, 'pool>(&'local self, store: &'store C::Store<'pool>) -> Self::Out;
}

impl<C: Compiler> Pretty<C> for Type<C> {
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

impl<C: Compiler> Pretty<C> for Qualified<C> {
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

pub fn print_function_reference<'local, 'store, 'pool, C: Compiler>(function_reference: &'local C::FunctionReference, store: &'store C::Store<'pool>) -> String {
  let function = function_reference.rget_from(store);
  let path = store.describe_module(function.parent);
  let name = function.header.name.print(store);
  format!("{path}::{name}")
}

impl<C: Compiler> Pretty<C> for ExpressionReference<C> {
  type Out = String;

  fn print<'local, 'store, 'pool>(&self, store: &'store C::Store<'pool>) -> Self::Out {
    let Span { start, end, .. } = self.rget_from(store).get_span(store);

    format!("expr {}:{} - {}:{}", start.line, start.column, end.line, end.column)
  }
}

impl<C: Compiler> Pretty<C> for BlockReference<C> {
  type Out = String;

  fn print<'local, 'store, 'pool>(&self, store: &'store C::Store<'pool>) -> Self::Out {
    let Span { start, end, .. } = self.rget_from(store).span;
    format!("block {}:{} - {}:{}", start.line, start.column, end.line, end.column)
  }
}

impl<C: Compiler> Pretty<C> for TypeReference<C> {
  type Out = String;

  fn print<'local, 'store, 'pool>(&self, store: &'store C::Store<'pool>) -> Self::Out {
    match self {
      TypeReference::Part(part) => part.rget_from(store).print(store),
      TypeReference::ReturnTypeOf(function) => {
        format!("ReturnType<{}>", print_function_reference::<C>(function, store))
      },
      TypeReference::Alias(alias ) => {
        let path = store.describe_module(alias.0);
        let name = alias.rget_from(store).name.print(store);
        format!("{path}::{name}")
      },
      TypeReference::Variable(VariableReference::Argument(parent, index)) => {
        format!("ArgumentOf<{}>[{index}]", print_function_reference::<C>(parent, store))
      },
      TypeReference::Variable(v @ VariableReference::Block(block, _)) => {
        let name = v.rget_from(store).name;

        format!("typeof {{{} {}}}::{}", print_function_reference::<C>(&v.parent(), store), block.print(store), name.print(store))
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

type FunctionAnd<'store, C, T> = (&'store Function<C>, &'store T);

pub trait PrettyFunction<'store, C: Compiler + 'store>: Sized + 'store where FunctionAnd<'store, C, Self>: Pretty<C> {
  fn print_with<'local, 'pool>(&'local self, function: &'store Function<C>, store: &'store C::Store<'pool>) -> <FunctionAnd<'store, C, Self> as Pretty<C>>::Out where 'local: 'store;
}

impl<'store, C: Compiler + 'store, T: 'store> PrettyFunction<'store, C> for T where FunctionAnd<'store, C, T>: Pretty<C> {
  fn print_with<'local, 'pool>(&'local self, function: &'store Function<C>, store: &'store C::Store<'pool>) -> <FunctionAnd<'store, C, Self> as Pretty<C>>::Out where 'local: 'store {
    (function, self).print(store)
  }
}

impl<'store_a, C: Compiler> Pretty<C> for FunctionAnd<'store_a, C, Expression<C>> {
  type Out = std::vec::IntoIter<String>;

  fn print<'local, 'store, 'pool>(&'local self, store: &'store C::Store<'pool>) -> Self::Out {
    let (function, expression) = self;
    match expression {
      Expression::Block(block_id) => {
        block_id.rget_from(store).print_with(function, store)
      },
      Expression::Literal { value, out, .. } => {
        let ty = out.print(store);
        vec![match value {
          LiteralKind::Numeric(NumericValue::F64(value)) => {
            format!("{value} {{{ty}}}")
          },
          LiteralKind::Numeric(NumericValue::U64(value)) => {
            format!("{value} {{{ty}}}")
          },
          LiteralKind::String { value, kind } => {
            let prefix = match kind {
              StringKind::Wide => "",
              StringKind::Byte => "b",
              StringKind::C => "c",
            };

            format!(
              "{prefix}{value:?} {{{ty}}}",
              value = unsafe { store.pool().get_string(*value) },
            )
          },
        }].into_iter()
      },
      Expression::Variable { reference, .. } => vec![
        reference.rget_from(store).name.print(store)
      ].into_iter(),
      Expression::Unary { expr, op, .. } => {
        let expr = expr.rget_from(store).print_with(function, store).collect::<String>();

        vec![match &op.0 {
          UnaryOperator::Prefix(prefix) => match prefix {
            UnaryPrefixOperator::Deref => format!("{{ *{expr} }}"),
            UnaryPrefixOperator::Ref => format!("{{ &{expr} }}"),
            UnaryPrefixOperator::MutRef => format!("{{ &mut {expr} }}"),
            UnaryPrefixOperator::Not => format!("{{ !{expr} }}"),
            UnaryPrefixOperator::Invert => format!("{{ ~{expr} }}"),
            UnaryPrefixOperator::Identity => format!("{{ +{expr} }}"),
            UnaryPrefixOperator::Negate => format!("{{ -{expr} }}"),
            UnaryPrefixOperator::PreDecrement => format!("{{ --{expr} }}"),
            UnaryPrefixOperator::PreIncrement => format!("{{ ++{expr} }}"),
            UnaryPrefixOperator::Splat => format!("{{ ...{expr} }}"),
          },
          UnaryOperator::Suffix(suffix) => match suffix {
            UnarySuffixOperator::Try => format!("{{ {expr}? }}"),
            UnarySuffixOperator::Call(exprs) => {
              let exprs = exprs.iter().map(|expr| {
                expr.rget_from(store)
                  .print_with(function, store)
                  .collect::<Vec<String>>()
                  .join("\n    ")
              });

              let args = exprs.collect::<Vec<_>>().join(", ");

              format!("{{ {expr}({args}) }}")
            },
            UnarySuffixOperator::PostDecrement => format!("{{ {expr}-- }}"),
            UnarySuffixOperator::PostIncrement => format!("{{ {expr}++ }}"),
          },
        }].into_iter()
      }
      Expression::Binary { a, b, op, .. } => {
        let a = a.rget_from(store).print_with(function, store).collect::<Vec<String>>().join("\n    ");
        let b = b.rget_from(store).print_with(function, store).collect::<Vec<String>>().join("\n    ");
        let op = &op.0;

        vec![format!("{{ {a} {op} {b} }}")].into_iter()
      },
      Expression::StructInitializer { ty, members, .. } => {
        let mut lines = vec![
          format!("{} {{", ty.print(store)),
        ];

        for (name, value) in members.iter() {
          let name = name.print(store);
          let mut expr_line_iter = value.rget_from(store).print_with(function, store);

          let first_line = expr_line_iter.next().unwrap();

          lines.push(format!("  {name}: {first_line}"));

          for rest_line in expr_line_iter {
            lines.push(format!("    {rest_line}"));
          };
        };

        if members.is_empty() {
          *lines.first_mut().unwrap() += "}";
        } else {
          lines.push("}".into());
        };

        lines.into_iter()
      },
    }
  }
}

impl<C: Compiler> Pretty<C> for FunctionAnd<'_, C, BlockExpression<C>> {
  type Out = std::vec::IntoIter<String>;

  fn print<'local, 'store, 'pool>(&'local self, store: &'store C::Store<'pool>) -> Self::Out {
    let (function, block) = self;
    let mut lines = vec!["{".into()];

    for variable in block.variables.iter() {
      let ty = variable.ty.print(store);
      let name = variable.name.print(store);

      lines.push(format!("  {ty} {name}"));
    };

    if !block.variables.is_empty() {
      lines.push("".into());
    };

    for &child in block.children.iter() {
      for line in function[child].print_with(function, store) {
        lines.push(format!("  {line}"));
      };
    };

    if lines.len() == 1 {
      lines.first_mut().unwrap().push('}');
    } else {
      if !block.returns_last {
        lines.last_mut().unwrap().push(';');
      };

      lines.push("}".into());
    };

    lines.into_iter()
  }
}

impl<C: Compiler> Pretty<C> for Name<C> {
  type Out = String;

  fn print<'local, 'store, 'pool>(&'local self, store: &'store C::Store<'pool>) -> Self::Out {
    store.pool().get(self.id)
  }
}

impl<C: Compiler> Pretty<C> for Function<C> {
  type Out = std::vec::IntoIter<String>;

  fn print<'local, 'store, 'pool>(&'local self, store: &'store C::Store<'pool>) -> Self::Out {
    let mut lines = vec![];
    let mut first: String = self.header.name.print(store);

    let ret_ty = self.header.ret_ty.print(store);
    first.push_str(format!(" -> {ret_ty}").as_str());

    lines.push(first);

    for argument in self.header.arguments.iter() {
      let ty = argument.ty.print(store);
      let name = argument.name.print(store);
      lines.push(format!("  {ty} {name}"));
    };

    lines.push("".into());
    let block = store.rget(self.body);

    for variable in block.variables.iter() {
      let ty = variable.ty.print(store);
      let name = variable.name.print(store);

      lines.push(format!("  {ty} {name} // decl"));
    };

    if !block.variables.is_empty() {
      lines.push("".into());
    };

    if !block.children.is_empty() {
      for &child in block.children.iter() {
        for line in self[child].print_with(self, store) {
          lines.push(format!("  {line}"));
        };
      };

      if !block.returns_last {
        lines.last_mut().unwrap().push(';');
      };
    } else {
      lines.push("{}".into());
    };

    lines.into_iter()
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

impl<C: Compiler> Pretty<C> for Module<C> {
  type Out = std::vec::IntoIter<String>;

  fn print<'local, 'store, 'pool>(&self, store: &'store C::Store<'pool>) -> Self::Out {
    let name = store.pool().get(self.name);
    let mut lines = vec![
      format!("mod {name}")
    ];

    let mut needs_empty = false;

    for (key, value) in self.transports.import_map.iter() {
      if needs_empty {
        lines.push("".into());
      };

      lines.push(format!("  import from {:?}", value.implicit.print(store)));
      lines.push(format!("    {} // {}", store.pool().get(*key), value.print(store)));

      needs_empty = true;
    };

    for (space, _) in self.transports.import_stars.iter() {
      if needs_empty {
        lines.push("".into());
      };

      lines.push(format!("  import from {}", space.print(store)));
      lines.push("    *".into());

      needs_empty = true;
    };

    if needs_empty {
      lines.push("".into());
      needs_empty = false;
    };

    for alias in self.aliases.iter() {
      let name = alias.name.print(store);
      let ty = alias.ty.print(store);
      lines.push(format!("  type {name} := {ty}"));

      needs_empty = true;
    };

    for struc in self.structs.iter() {
      if needs_empty {
        lines.push("".into());
      };

      let name = struc.name.print(store);

      lines.push(format!("  struct {name}"));

      for member in struc.members.iter() {
        lines.push(format!("    {} {}", member.ty.print(store), member.name.print(store)));
      };

      needs_empty = true;
    };

    for &module in self.modules.iter() {
      let module_borrow = store.rget(module);

      if needs_empty {
        lines.push("".into());
      };

      lines.push(format!("  // {}", store.describe_module(module)));
      for line in module_borrow.print(store) {
        lines.push(format!("  {line}"));
      };

      needs_empty = true;
    };

    for function in self.functions.iter() {
      let function_borrow = function.rget_from(store);

      if needs_empty {
        lines.push("".into());
      };

      for line in function_borrow.print(store) {
        lines.push(format!("  {line}"));
      };

      needs_empty = true;
    };

    lines.into_iter()
  }
}
