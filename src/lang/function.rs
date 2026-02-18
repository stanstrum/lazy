use std::ops::{Index, IndexMut};

use crate::lang::expr::{BlockExpression, Expression};
use crate::lang::module::Name;
use crate::lang::reference::{BlockReference, FunctionReference, ModuleReference};
use crate::lang::ty::Type;
use crate::tokenize::token::Span;

#[derive(Debug)]
pub struct FunctionArgument {
  pub name: Name,
  pub ty: Type,
  pub span: Span,
}

#[derive(Debug)]
pub struct FunctionHeader {
  pub name: Name,
  pub ret_ty: Type,
  pub arguments: Vec<FunctionArgument>,
  pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExprId(usize);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockId(usize);

#[derive(Debug)]
pub struct Function {
  pub parent: ModuleReference,
  pub header: FunctionHeader,
  pub body: BlockReference,
  pub blocks: Vec<BlockExpression>,
  pub exprs: Vec<Expression>,
  pub span: Span,
}

impl Function {
  pub fn new(function: FunctionReference, parent: ModuleReference, header: FunctionHeader) -> Self {
    let temp_span = header.span;
    let blocks = vec![BlockExpression::new_dirty(temp_span)];
    let body = BlockReference(function, BlockId(0));

    Self {
      parent,
      header,
      body,
      blocks,
      exprs: vec![],
      span: temp_span,
    }
  }

  pub fn add_block(&mut self, block: BlockExpression) -> BlockId {
    let id = BlockId(self.blocks.len());
    self.blocks.push(block);

    id
  }

  pub fn add_expr(&mut self, expr: Expression) -> ExprId {
    let id = ExprId(self.exprs.len());
    self.exprs.push(expr);

    id
  }

  pub fn add_expr_to_block(&mut self, expr: Expression, BlockReference(function, block): BlockReference) -> ExprId {
    assert!(function == self.body.0,
      "cannot add an expression using another function's BlockReference",
    );

    let id = self.add_expr(expr);
    self[block].children.push(id);

    id
  }
}

impl Index<BlockId> for Function {
  type Output = BlockExpression;

  fn index(&self, BlockId(index): BlockId) -> &Self::Output {
    self.blocks.get(index).unwrap()
  }
}

impl IndexMut<BlockId> for Function {
  fn index_mut(&mut self, BlockId(index): BlockId) -> &mut Self::Output {
    self.blocks.get_mut(index).unwrap()
  }
}

impl Index<ExprId> for Function {
  type Output = Expression;

  fn index(&self, ExprId(index): ExprId) -> &Self::Output {
    self.exprs.get(index).unwrap()
  }
}

impl IndexMut<ExprId> for Function {
  fn index_mut(&mut self, ExprId(index): ExprId) -> &mut Self::Output {
    self.exprs.get_mut(index).unwrap()
  }
}
