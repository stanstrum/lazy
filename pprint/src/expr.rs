use lang::token::{NumericValue, StringKind, UnaryOperator, UnaryPrefixOperator, UnarySuffixOperator};
use lang::expr::{Expression, LiteralKind};

use super::*;

impl<C: Compiler> Pretty<C> for lang::reference::ExpressionReference<C> {
  type Out = String;

  fn print<'local, 'store, 'pool>(&self, store: &'store C::Store<'pool>) -> Self::Out {
    let Span { start, end, .. } = self.rget_from(store).get_span(store);

    format!("expr {}:{} - {}:{}", start.line, start.column, end.line, end.column)
  }
}

impl<'store_a, C: Compiler> Pretty<C> for FunctionAnd<'store_a, C, lang::expr::Expression<C>> {
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
