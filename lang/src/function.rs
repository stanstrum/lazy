use std::ops::{Index, IndexMut};

use crate::Compiler;
use crate::span::Span;
use crate::module::Name;
use crate::reference::BlockReference;
use crate::ty::Type;
use crate::expr::{BlockExpression, Expression, Variable};

#[derive(Debug)]
pub struct FunctionHeader<C: Compiler> {
  pub name: Name<C>,
  pub ret_ty: Type<C>,
  pub arguments: Vec<Variable<C>>,
  pub span: Span<C>,
}

#[derive(Debug)]
pub struct Function<C: Compiler> {
  pub parent: C::ModuleReference,
  pub header: FunctionHeader<C>,
  pub body: BlockReference<C>,
  pub blocks: Vec<BlockExpression<C>>,
  pub exprs: Vec<Expression<C>>,
  pub span: Span<C>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExprId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockId(usize);

impl BlockId {
  /// We're using the assumption that the body is block that is created first,
  /// specifically at instantiation.
  pub const fn body_id() -> Self {
    Self(0)
  }
}

impl<C: Compiler> Function<C> {
  pub fn new(body: BlockReference<C>, parent: C::ModuleReference, header: FunctionHeader<C>) -> Self {
    let temp_span = header.span;
    let blocks = vec![BlockExpression::new_dirty(None, temp_span)];

    Self {
      parent,
      header,
      body,
      blocks,
      exprs: vec![],
      span: temp_span,
    }
  }

  pub fn add_block(&mut self, block: BlockExpression<C>) -> BlockId {
    let id = BlockId(self.blocks.len());
    self.blocks.push(block);

    id
  }

  pub fn add_expr(&mut self, expr: Expression<C>) -> ExprId {
    let id = ExprId(self.exprs.len());
    self.exprs.push(expr);

    id
  }

  // pub fn add_expr_to_block(&mut self, expr: Expression, BlockReference(function, block): BlockReference) -> ExprId {
  //   assert!(function == self.body.0,
  //     "cannot add an expression using another function's BlockReference",
  //   );

  //   let id = self.add_expr(expr);
  //   self[block].children.push(id);

  //   id
  // }
}

impl<C: Compiler> Index<BlockId> for Function<C> {
  type Output = BlockExpression<C>;

  fn index(&self, BlockId(index): BlockId) -> &Self::Output {
    self.blocks.get(index).unwrap()
  }
}

impl<C: Compiler> IndexMut<BlockId> for Function<C> {
  fn index_mut(&mut self, BlockId(index): BlockId) -> &mut Self::Output {
    self.blocks.get_mut(index).unwrap()
  }
}

impl<C: Compiler> Index<ExprId> for Function<C> {
  type Output = Expression<C>;

  fn index(&self, ExprId(index): ExprId) -> &Self::Output {
    self.exprs.get(index).unwrap()
  }
}

impl<C: Compiler> IndexMut<ExprId> for Function<C> {
  fn index_mut(&mut self, ExprId(index): ExprId) -> &mut Self::Output {
    self.exprs.get_mut(index).unwrap()
  }
}
