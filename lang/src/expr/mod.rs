pub mod operator;

use string_pool::StringId;

use crate::token::{NumericValue, StringKind};
use crate::span::Span;
use crate::reference::{BlockReference, ExpressionReference, Store, TypeReference, VariableReference};
use crate::ty::{Qualified, Type, TypeValue};
use crate::module::Name;
use crate::function::{BlockId, ExprId};
use crate::Compiler;

#[derive(Debug)]
pub struct Variable<C: Compiler> {
  pub name: Name<C>,
  pub ty: Type<C>,
  pub span: Span<C>,
}

#[derive(Debug)]
pub struct BlockExpression<C: Compiler> {
  pub parent: Option<BlockReference<C>>,
  pub children: Vec<ExprId>,
  pub span: Span<C>,
  pub returns_last: bool,
  pub out: Type<C>,
  pub variables: Vec<Variable<C>>,
}

#[derive(Debug, Clone, Copy)]
pub enum LiteralKind {
  Numeric(NumericValue),
  String {
    value: StringId,
    kind: StringKind,
  },
}

#[derive(Debug)]
pub enum Expression<C: Compiler> {
  Block(BlockReference<C>),
  Literal {
    value: LiteralKind,
    span: Span<C>,
    out: Type<C>,
  },
  Variable {
    reference: VariableReference<C>,
    span: Span<C>,
  },
  Unknown {
    qualified: Qualified<C>,
    out: Type<C>,
  },
  Unary {
    expr: ExpressionReference<C>,
    op: (operator::UnaryOperator<C>, Span<C>),
    span: Span<C>,
    out: Type<C>,
  },
  Binary {
    a: ExpressionReference<C>,
    b: ExpressionReference<C>,
    op: (operator::BinaryOperator, Span<C>),
    span: Span<C>,
    out: Type<C>,
  },
  StructInitializer {
    ty: Type<C>,
    members: Vec<(Name<C>, ExpressionReference<C>)>,
    span: Span<C>,
  },
}

impl<C: Compiler> Expression<C> {
  pub fn new_unknown(qualified: Qualified<C>) -> Self {
    let span = qualified.span;

    Self::Unknown {
      qualified,
      out: todo!(),
      // TypeValue::Weak { span },
    }
  }
}

impl<C: Compiler> BlockExpression<C> {
  pub fn new(parent: Option<BlockReference<C>>, span: Span<C>, out: Type<C>) -> Self {
    Self {
      parent,
      children: vec![],
      span,
      returns_last: false,
      out,
      variables: vec![],
    }
  }

  pub fn create_in(
    store: &mut C::Store<'_>,
    function_reference: C::FunctionReference,
    parent: Option<BlockReference<C>>,
    span: Span<C>,
    value: TypeValue<C>,
  ) -> BlockReference<C> {
    let function_borrow = store.rget_mut(function_reference);

    // SPONGE: this needs to be done cleaner
    let next_block_id = BlockId(function_borrow.blocks.len());
    let next_block_reference = BlockReference(function_reference, next_block_id);

    // This is completely incompatible with multithreading, just for starts
    let out_ty_reference = TypeReference::Block(next_block_reference);
    let out = Type::new(out_ty_reference, value);

    let block = BlockExpression::new(parent, span, out);

    let legacy_block_id = function_borrow.add_block(block);

    assert!(legacy_block_id == next_block_reference.1,
      "disagreement over where this newly created block expr is!");

    next_block_reference
  }

  pub fn create_weak_in(
    store: &mut C::Store<'_>,
    function_reference: C::FunctionReference,
    parent: Option<BlockReference<C>>,
    span: Span<C>,
  ) -> BlockReference<C> {
    Self::create_in(store, function_reference, parent, span, TypeValue::Weak { span })
  }

  /// "New Contextualized" - creates an [`Expression`] using a callback to
  /// instantiate the expr with its future [`ExpressionReference`]
  /// -- not thread safe
  pub fn create_new_expr_in(
    store: &mut C::Store<'_>,
    block_reference: BlockReference<C>,
    cb: impl FnOnce(ExpressionReference<C>) -> Expression<C>,
  ) -> ExpressionReference<C> {
    let function_reference = block_reference.0;
    let function_borrow = store.rget_mut(function_reference);

    let next_expr_id = ExprId(function_borrow.exprs.len());
    let next_expression_reference = ExpressionReference(block_reference, next_expr_id);

    let expr = cb(next_expression_reference);

    let legacy_expr_id = function_borrow.add_expr(expr);

    assert!(legacy_expr_id == next_expr_id, "disagreement over where this expr is!");
    store.rget_mut(block_reference).children.push(next_expr_id);

    next_expression_reference
  }
}
