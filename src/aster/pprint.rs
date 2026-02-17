use crate::lang::expr::operator::{UnaryOperator, UnarySuffixOperator};
use crate::lang::expr::{BlockExpression, Expression, LiteralKind};
use crate::lang::reference::{Reference, Store, TypePartReference};
use crate::string_pool::PoolId;

use crate::lang::Lazy;
use crate::lang::ty::{Qualified, Type};
use crate::lang::module::{Module, Name};
use crate::lang::function::Function;
use crate::tokenize::token::{NumericValue, StringKind};

pub trait Pretty {
  type Out;

  fn print(&self, lazy: &Lazy) -> Self::Out;
}

impl Pretty for PoolId {
  type Out = String;

  fn print(&self, lazy: &Lazy) -> Self::Out {
    lazy.pool.get(*self).collect()
  }
}

// impl Pretty for TypeReference {
//   type Out = String;

//   fn print(&self, lazy: &Lazy) -> Self::Out {
//     self.rget_from(lazy).print(lazy)
//   }
// }

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

    if self.implicit {
      out += "::";
    };

    for (i, part) in self.parts.iter().enumerate() {
      if i != 0 {
        out += "::";
      };

      out += part.id.print(lazy).as_str();
    };

    out
  }
}

impl Pretty for Type {
  type Out = String;

  fn print(&self, lazy: &Lazy) -> Self::Out {
    match self {
      // Type::Reference(reference) => reference.print(lazy),
      Type::Unresolved { qualified, .. } => qualified.print(lazy),
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
      Type::ReferenceTo { ty, .. } => format!("&{}", ty.print(lazy)),
      Type::SizedArrayOf { ty, size, .. } => format!("[{size}]{}", ty.print(lazy)),
      Type::UnsizedArrayOf { ty, .. } => format!("[]{}", ty.print(lazy)),
      // Type::Expression(expression) => {
      //   let fname = lazy[expression.function].header.name.print(lazy);
      //   let index = expression.index;
      //   format!("/* typeof {fname}:{index:?} */")
      // },
      other => todo!("{other:?}"),
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
            format!("{value} /* {ty} */")
          },
          LiteralKind::Numeric(NumericValue::U64(value)) => {
            format!("{value} /* {ty} */")
          },
          LiteralKind::String { value, kind } => {
            let prefix = match kind {
              StringKind::Wide => "",
              StringKind::Byte => "b",
              StringKind::C => "c",
            };

            let value = lazy.pool.get_string(*value);
            format!("{prefix}{value:?} /* {ty} */")
          },
        }].into_iter()
      },
      Expression::Variable { reference, .. } => vec![
        reference.rget_from(lazy).name.print(lazy)
      ].into_iter(),
      Expression::Unknown(qualified) => vec![
        qualified.print(lazy)
      ].into_iter(),
      Expression::Unary { expr, op, .. } => {
        let expr = expr.rget_from(lazy).print_with(function, lazy).collect::<String>();

        vec![match &op.0 {
          UnaryOperator::Prefix(prefix) => todo!(),
          UnaryOperator::Suffix(suffix) => match suffix {
            UnarySuffixOperator::Try => format!("{expr}?"),
            UnarySuffixOperator::Call(exprs) => {
              let exprs = exprs.iter().map(|expr| {
                expr.rget_from(lazy)
                  .print_with(function, lazy)
                  .collect::<String>()
              });

              let args = exprs.collect::<Vec<_>>().join(", ");

              format!("{expr}(args)")
            },
            UnarySuffixOperator::PostDecrement => format!("{expr}--"),
            UnarySuffixOperator::PostIncrement => format!("{expr}++"),
          },
        }].into_iter()
      }
      other => todo!("{other:#?}"),
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

impl Pretty for Module {
  type Out = std::vec::IntoIter<String>;

  fn print(&self, lazy: &Lazy) -> Self::Out {
    let name = self.name.print(lazy);
    let mut lines = vec![
      format!("namespace {name}")
    ];

    for alias in self.aliases.iter() {
      let name = alias.name.print(lazy);
      let ty = alias.ty.print(lazy);
      lines.push(format!("  type {name} := {ty}"));
    };

    for (i, &module) in self.modules.iter().enumerate() {
      let module_ref = lazy.rget(module);

      if i != 0 {
        lines.push("".into());
      };

      lines.push(format!("  /* {} */", lazy.describe_module(module)));
      for line in module_ref.print(lazy) {
        lines.push(format!("  {line}"));
      };
    };

    for (i, &function) in self.functions.iter().enumerate() {
      let function_ref = lazy.rget(function);

      if i != 0 {
        lines.push("".into());
      };

      for line in function_ref.print(lazy) {
        lines.push(format!("  {line}"));
      };
    };

    lines.into_iter()
  }
}
