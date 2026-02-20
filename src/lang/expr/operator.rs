use super::*;

#[derive(Debug, Clone, Copy)]
pub enum BinaryOperator {
  Add, // +
  Sub, // -
  Mul, // *
  Div, // /
  Mod, // %
  Exp, // **
  And, // &
  Or, // |
  Xor, // ^
  Shr, // >>
  Shl, // <<
  LogicalAnd, // &&
  LogicalOr, // ||
  LogicalXor, // ^^
  LogicalShr, // >>>
  DerefDot, // ->
  Dot, // .

  Assign, // =
  AddAssign, // +=
  SubAssign, // -=
  MulAssign, // *=
  DivAssign, // /=
  ModAssign, // %=
  ExpAssign, // **=
  AndAssign, // &=
  OrAssign, // |=
  XorAssign, // ^=
  ShlAssign, // >>=
  ShrAssign, // <<=
  LogicalAndAssign, // &&=
  LogicalOrAssign, // ||=
  LogicalXorAssign, // ^^=
  LogicalShrAssign, // >>>=

  Less, // <
  LessEqual, // <=
  Greater, // >
  GreaterEqual, // >=
  Equal, // ==

  Fish, // <>
  Range, // ..
  Splat, // ...
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryPrefixOperator {
  Deref,
  Ref,
  MutRef,
  Not,
  Invert,
  Identity,
  Negate,
  PreDecrement,
  PreIncrement,
  Splat,
}

#[derive(Debug, Clone)]
pub enum UnarySuffixOperator {
  Try,
  Call(Vec<ExpressionReference>),
  PostDecrement,
  PostIncrement,
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
  Prefix(UnaryPrefixOperator),
  Suffix(UnarySuffixOperator),
}

impl std::fmt::Display for BinaryOperator {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(match self {
      BinaryOperator::Add => "+",
      BinaryOperator::Sub => "-",
      BinaryOperator::Mul => "*",
      BinaryOperator::Div => "/",
      BinaryOperator::Mod => "%",
      BinaryOperator::Exp => "**",
      BinaryOperator::And => "&",
      BinaryOperator::Or => "|",
      BinaryOperator::Xor => "^",
      BinaryOperator::Shr => ">>",
      BinaryOperator::Shl => "<<",
      BinaryOperator::LogicalAnd => "&&",
      BinaryOperator::LogicalOr => "||",
      BinaryOperator::LogicalXor => "^^",
      BinaryOperator::LogicalShr => ">>>",
      BinaryOperator::Dot => ".",
      BinaryOperator::DerefDot => "->",
      BinaryOperator::Assign => "=",
      BinaryOperator::AddAssign => "+=",
      BinaryOperator::SubAssign => "+-",
      BinaryOperator::MulAssign => "*=",
      BinaryOperator::DivAssign => "/=",
      BinaryOperator::ModAssign => "%=",
      BinaryOperator::ExpAssign => "**=",
      BinaryOperator::AndAssign => "&=",
      BinaryOperator::OrAssign => "|=",
      BinaryOperator::XorAssign => "^=",
      BinaryOperator::ShlAssign => "<<=",
      BinaryOperator::ShrAssign => ">>=",
      BinaryOperator::LogicalAndAssign => "&&=",
      BinaryOperator::LogicalOrAssign => "||=",
      BinaryOperator::LogicalXorAssign => "^^=",
      BinaryOperator::LogicalShrAssign => ">>>=",
      BinaryOperator::Less => "<",
      BinaryOperator::LessEqual => "<=",
      BinaryOperator::Greater => ">",
      BinaryOperator::GreaterEqual => ">=",
      BinaryOperator::Equal => "==",
      BinaryOperator::Fish => "<>",
      BinaryOperator::Range => "..",
      BinaryOperator::Splat => "...",
    })
  }
}
