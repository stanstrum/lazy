use super::*;

#[derive(Debug)]
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
  Dot, // .

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
  ShrAsign, // <<=
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

#[derive(Debug)]
pub enum UnaryPrefixOperator {
  Deref,
  Ref,
  MutRef,
  Not,
  Invert,
}

#[derive(Debug)]
pub enum UnarySuffixOperator {
  Try,
  Call(Vec<ExpressionReference>),
}

#[derive(Debug)]
pub enum UnaryOperator {
  Prefix(UnaryPrefixOperator),
  Suffix(UnarySuffixOperator),
}
