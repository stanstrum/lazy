use std::ops::{Index, IndexMut};

use crate::lang::expr::BlockExpression;
use crate::lang::module::ModuleId;
use crate::lang::ty::Type;
use crate::tokenize::token::Span;
use crate::string_pool::PoolId;

#[derive(Debug)]
pub struct FunctionArgument {
  pub name: PoolId,
  pub ty: Type,
  pub span: Span,
}

#[derive(Debug)]
pub struct FunctionHeader {
  pub name: PoolId,
  pub ret_ty: Type,
  pub arguments: Vec<FunctionArgument>,
  pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub struct BlockId(usize);

#[derive(Debug)]
pub struct Function {
  pub parent: ModuleId,
  pub header: FunctionHeader,
  pub body: BlockId,
  pub blocks: Vec<BlockExpression>,
  pub span: Span,
}

impl Function {
  pub fn new(parent: ModuleId, header: FunctionHeader) -> (Self, BlockId) {
    let temp_span = header.span;
    let blocks = vec![BlockExpression::new(temp_span)];
    let body = BlockId(0);

    let function = Self {
      parent,
      header,
      body,
      blocks,
      span: temp_span,
    };

    (function, body)
  }

  pub fn add_block(&mut self, block: BlockExpression) -> BlockId {
    let id = BlockId(self.blocks.len());
    self.blocks.push(block);

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
