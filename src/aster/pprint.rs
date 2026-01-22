use crate::lang::expr::{BlockExpression, Expression};
use crate::string_pool::PoolId;

use crate::lang::Lazy;
use crate::lang::ty::Type;
use crate::lang::module::Module;
use crate::lang::function::Function;
use crate::tokenize::token::NumericValue;

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

impl Pretty for Type {
  type Out = String;

  fn print(&self, lazy: &Lazy) -> Self::Out {
    match self {
      Type::Unresolved { qualified, .. } => {
        qualified.parts.iter()
          .map(|x| x.print(lazy))
          .collect::<Vec<_>>()
          .join("::")
      },
      Type::Intrinsic { kind, .. } => kind.to_string(),
      Type::Deferred(_) => todo!(),
      Type::WeakFloat { .. } => "{float}".into(),
      Type::WeakInteger { .. } => "{weak integer}".into(),
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
      Expression::BlockExpression(block_id) => {
        function[*block_id].print_with(function, lazy)
      },
      Expression::Literal { value: NumericValue::F64(value), .. } => {
        vec![format!("{value}")].into_iter()
      },
      Expression::Literal { value: NumericValue::U64(value), .. } => {
        vec![format!("{value}")].into_iter()
      },
    }
  }
}

impl Pretty for FunctionAnd<'_, BlockExpression> {
  type Out = std::vec::IntoIter<String>;

  fn print(&self, lazy: &Lazy) -> Self::Out {
    let (function, block) = self;
    let mut lines = vec!["{".into()];

    for child in block.children.iter() {
      for line in child.print_with(function, lazy) {
        lines.push(format!("  {line}"));
      };
    };

    if lines.len() == 1 {
      lines.first_mut().unwrap().push('}');
    } else {
      lines.push("}".into());
    };

    lines.into_iter()
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
    let block = &self[self.body];
    if !block.children.is_empty() {
      for child in block.children.iter() {
        for line in child.print_with(self, lazy) {
          lines.push(format!("  {line}"));
        };
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

    for (i, &id) in self.modules.iter().enumerate() {
      let module = &lazy[id];

      if i != 0 {
        lines.push("".into());
      };

      lines.push(format!("  /* {} */", lazy.describe_module(id)));
      for line in module.print(lazy) {
        lines.push(format!("  {line}"));
      };
    };

    for (i, &id) in self.functions.iter().enumerate() {
      let function = &lazy[id];

      if i != 0 {
        lines.push("".into());
      };

      for line in function.print(lazy) {
        lines.push(format!("  {line}"));
      };
    };

    lines.into_iter()
  }
}
