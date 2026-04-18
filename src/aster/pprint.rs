use crate::lang::expr::operator::{UnaryOperator, UnaryPrefixOperator, UnarySuffixOperator};
use crate::lang::expr::{BlockExpression, Expression, LiteralKind};
use crate::lang::reference::{BlockReference, ExpressionReference, FunctionReference, Reference, Store, TypePartReference, TypeReference, VariableReference};
use crate::lang::span::GetSpan;
use crate::resolve::{TypeOf, TypePair, TypePairModifier};
use crate::string_pool::PoolId;

use crate::lang::Lazy;
use crate::lang::ty::{Qualified, QualifiedSearchSpace, Type};
use crate::lang::module::{Module, Name};
use crate::lang::function::Function;
use crate::tokenize::token::{NumericValue, Span, StringKind};

pub trait Pretty {
  type Out;

  fn print(&self, lazy: &Lazy) -> Self::Out;
}

impl Pretty for PoolId {
  type Out = String;

  fn print(&self, lazy: &Lazy) -> Self::Out {
    lazy.pool.get(*self)
  }
}

impl Pretty for TypePair {
  type Out = String;

  fn print(&self, lazy: &Lazy) -> Self::Out {
    let reference = self.reference.print(lazy);

    let mut out = format!("/* {{pair}} */ {reference}");

    for modifier in self.modifiers.iter() {
      match modifier {
        TypePairModifier::Dereference => out = format!("Dereference<{out}>"),
      };
    };

    out
  }
}

impl Pretty for TypePartReference {
  type Out = String;

  fn print(&self, lazy: &Lazy) -> Self::Out {
    self.rget_from(lazy).print(lazy)
  }
}

impl Pretty for Qualified {
  type Out = String;

  fn print(&self, lazy: &Lazy) -> Self::Out {
    let mut out = String::new();

    match &self.implicit {
      QualifiedSearchSpace::Implicit => {},
      &QualifiedSearchSpace::Module(module_reference) => {
        out += &lazy.describe_module(module_reference);
      },
      QualifiedSearchSpace::Intrinsic { kind, .. } => {
        out += &kind.to_string();
      }
      QualifiedSearchSpace::Type(type_reference) => {
        out += &type_reference.print(lazy);
      },
    };

    out += "::";

    for (i, part) in self.parts.iter().enumerate() {
      if i != 0 {
        out += "::";
      };

      out += part.id.print(lazy).as_str();
    };

    out
  }
}

impl Pretty for FunctionReference {
  type Out = String;

  fn print(&self, lazy: &Lazy) -> Self::Out {
    let function = self.rget_from(lazy);
    let path = lazy.describe_module(function.parent);
    let name = function.header.name.print(lazy);

    format!("{path}::{name}")
  }
}

impl Pretty for ExpressionReference {
  type Out = String;

  fn print(&self, lazy: &Lazy) -> Self::Out {
    let Span { start, end, .. } = self.rget_from(lazy).get_span(lazy);

    format!("expr {}:{} - {}:{}", start.line, start.column, end.line, end.column)
  }
}

impl Pretty for BlockReference {
  type Out = String;

  fn print(&self, lazy: &Lazy) -> Self::Out {
    let Span { start, end, .. } = self.rget_from(lazy).span;
    format!("block {}:{} - {}:{}", start.line, start.column, end.line, end.column)
  }
}

impl Pretty for TypeReference {
  type Out = String;

  fn print(&self, lazy: &Lazy) -> Self::Out {
    match self {
      TypeReference::Part(part) => part.rget_from(lazy).print(lazy),
      TypeReference::ReturnTypeOf(function) => {
        format!("ReturnType<{}>", function.print(lazy))
      },
      TypeReference::Alias(alias ) => {
        let path = lazy.describe_module(alias.0);
        let name = alias.rget_from(lazy).name.print(lazy);
        format!("{path}::{name}")
      },
      TypeReference::Variable(VariableReference::Argument(parent, index)) => {
        format!("ArgumentOf<{}>[{index}]", parent.print(lazy))
      },
      TypeReference::Variable(v @ VariableReference::Block(block, _)) => {
        let name = v.rget_from(lazy).name;

        format!("typeof {{{} {}}}::{}", v.parent().print(lazy), block.print(lazy), name.print(lazy))
      },
      TypeReference::Expression(expression) => {
        let type_print = expression.type_of(lazy).map(|s| format!(" /* {} */", s.print(lazy)));
        let type_print = type_print.as_deref().unwrap_or_default();

        format!("typeof {{{}}}{type_print}", expression.print(lazy))
      },
      TypeReference::Block(block) => format!("typeof {{{}}}", block.print(lazy)),
    }
  }
}

impl Pretty for Type {
  type Out = String;

  fn print(&self, lazy: &Lazy) -> Self::Out {
    match self {
      // Type::Reference(reference) => reference.print(lazy),
      Type::Unresolved { qualified, .. } => format!("{{unknown}} {}", qualified.print(lazy)),
      Type::Intrinsic { kind, .. } => kind.to_string(),
      // Type::Resolved { original, reference } => {
      //   format!("/* {deferred} */ {original}",
      //     deferred = reference.print(lazy),
      //     original = original.print(lazy),
      //   )
      // },
      Type::WeakFloat { .. } => "{weak float}".into(),
      Type::WeakInteger { .. } => "{weak integer}".into(),
      Type::WeakString { .. } => "{weak string}".into(),
      Type::ReferenceTo { ty, r#mut, .. } => format!("&{mutable}{ty}",
        mutable = if *r#mut { "mut " } else { "" },
        ty = ty.print(lazy),
      ),
      Type::SizedArrayOf { ty, size, .. } => format!("[{size}]{}", ty.print(lazy)),
      Type::UnsizedArrayOf { ty, .. } => format!("[]{}", ty.print(lazy)),
      // Type::Expression(expression) => {
      //   let fname = lazy[expression.function].header.name.print(lazy);
      //   let index = expression.index;
      //   format!("/* typeof {fname}:{index:?} */")
      // },
      Type::Resolved { part, .. } => format!("|{}|", part.print(lazy)),
      Type::Reference(reference) => format!("|{}|", reference.print(lazy)),
      Type::Weak { .. } => "{weak}".into(),
      // other => todo!("{other:?}"),
    }
  }
}

type FunctionAnd<'a, T> = (&'a Function, &'a T);

pub trait PrettyFunction<'a>: Sized where FunctionAnd<'a, Self>: Pretty + 'a {
  fn print_with(&'a self, function: &'a Function, lazy: &Lazy) -> <FunctionAnd<'a, Self> as Pretty>::Out;
}

impl<'a, T: 'a> PrettyFunction<'a> for T where FunctionAnd<'a, T>: Pretty {
  fn print_with(&'a self, function: &'a Function, lazy: &Lazy) -> <FunctionAnd<'a, Self> as Pretty>::Out {
    (function, self).print(lazy)
  }
}

impl Pretty for FunctionAnd<'_, Expression> {
  type Out = std::vec::IntoIter<String>;

  fn print(&self, lazy: &Lazy) -> Self::Out {
    let (function, expression) = self;
    match expression {
      Expression::Block(block_id) => {
        block_id.rget_from(lazy).print_with(function, lazy)
      },
      Expression::Literal { value, out, .. } => {
        let ty = out.print(lazy);
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
              value = unsafe { lazy.pool.get_string(*value) },
            )
          },
        }].into_iter()
      },
      Expression::Variable { reference, .. } => vec![
        reference.rget_from(lazy).name.print(lazy)
      ].into_iter(),
      Expression::Unknown { qualified, .. } => vec![
        format!("{{?}} {}", qualified.print(lazy))
      ].into_iter(),
      Expression::Unary { expr, op, .. } => {
        let expr = expr.rget_from(lazy).print_with(function, lazy).collect::<String>();

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
                expr.rget_from(lazy)
                  .print_with(function, lazy)
                  .collect::<String>()
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
        let a = a.rget_from(lazy).print_with(function, lazy).collect::<String>();
        let b = b.rget_from(lazy).print_with(function, lazy).collect::<String>();
        let op = &op.0;

        vec![format!("{{ {a} {op} {b} }}")].into_iter()
      },
    }
  }
}

impl Pretty for FunctionAnd<'_, BlockExpression> {
  type Out = std::vec::IntoIter<String>;

  fn print(&self, lazy: &Lazy) -> Self::Out {
    let (function, block) = self;
    let mut lines = vec!["{".into()];

    for variable in block.variables.iter() {
      let ty = variable.ty.print(lazy);
      let name = variable.name.print(lazy);

      lines.push(format!("  {ty} {name}"));
    };

    if !block.variables.is_empty() {
      lines.push("".into());
    };

    for &child in block.children.iter() {
      for line in function[child].print_with(function, lazy) {
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

impl Pretty for Name {
  type Out = String;

  fn print(&self, lazy: &Lazy) -> Self::Out {
    self.id.print(lazy)
  }
}

impl Pretty for Function {
  type Out = std::vec::IntoIter<String>;

  fn print(&self, lazy: &Lazy) -> Self::Out {
    let mut lines = vec![];
    let mut first: String = self.header.name.print(lazy);

    let ret_ty = self.header.ret_ty.print(lazy);
    first.push_str(format!(" -> {ret_ty}").as_str());

    lines.push(first);

    for argument in self.header.arguments.iter() {
      let ty = argument.ty.print(lazy);
      let name = argument.name.print(lazy);
      lines.push(format!("  {ty} {name}"));
    };

    lines.push("".into());
    let block = lazy.rget(self.body);

    for variable in block.variables.iter() {
      let ty = variable.ty.print(lazy);
      let name = variable.name.print(lazy);

      lines.push(format!("  {ty} {name} // decl"));
    };

    if !block.variables.is_empty() {
      lines.push("".into());
    };

    if !block.children.is_empty() {
      for &child in block.children.iter() {
        for line in self[child].print_with(self, lazy) {
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

impl Pretty for QualifiedSearchSpace {
  type Out = String;

  fn print(&self, lazy: &Lazy) -> Self::Out {
    match self {
      QualifiedSearchSpace::Implicit => "::".into(),
      QualifiedSearchSpace::Type(overwrite_type_reference) => overwrite_type_reference.print(lazy),
      QualifiedSearchSpace::Intrinsic { kind, .. } => kind.to_string(),
      QualifiedSearchSpace::Module(module_reference) => lazy.describe_module(*module_reference),
    }
  }
}

impl Pretty for Module {
  type Out = std::vec::IntoIter<String>;

  fn print(&self, lazy: &Lazy) -> Self::Out {
    let name = self.name.print(lazy);
    let mut lines = vec![
      format!("namespace {name}")
    ];

    let mut needs_empty = false;

    for (key, value) in self.transports.import_map.iter() {
      if needs_empty {
        lines.push("".into());
      };

      lines.push(format!("  import from {:?}", value.implicit.print(lazy)));
      lines.push(format!("    {} // {}", key.print(lazy), value.print(lazy)));

      needs_empty = true;
    };

    for (space, _) in self.transports.import_stars.iter() {
      if needs_empty {
        lines.push("".into());
      };

      lines.push(format!("  import from {}", space.print(lazy)));
      lines.push("    *".into());

      needs_empty = true;
    };

    if needs_empty {
      lines.push("".into());
      needs_empty = false;
    };

    for alias in self.aliases.iter() {
      let name = alias.name.print(lazy);
      let ty = alias.ty.print(lazy);
      lines.push(format!("  type {name} := {ty}"));

      needs_empty = true;
    };

    for &module in self.modules.iter() {
      let module_borrow = lazy.rget(module);

      if needs_empty {
        lines.push("".into());
      };

      lines.push(format!("  {{{}}}", lazy.describe_module(module)));
      for line in module_borrow.print(lazy) {
        lines.push(format!("  {line}"));
      };

      needs_empty = true;
    };

    for function in self.functions.iter() {
      let function_borrow = function.rget_from(lazy);

      if needs_empty {
        lines.push("".into());
      };

      for line in function_borrow.print(lazy) {
        lines.push(format!("  {line}"));
      };

      needs_empty = true;
    };

    lines.into_iter()
  }
}
