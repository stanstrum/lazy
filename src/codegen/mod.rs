/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

mod errors;

mod metadatatype;
mod namespace;
mod function;
mod expression;
mod r#type;
mod binding;
mod literal;
mod unary;

use std::collections::HashMap;

use errors::*;

use crate::aster::{
  ast::*,
  consts
};

use inkwell::{
  module::Module,
  context::Context,
  builder::Builder,
  values::AnyValueEnum
};

pub struct Codegen<'a, 'ctx> {
  pub context: &'ctx Context,
  pub module: &'a Module<'ctx>,
  pub builder: &'a Builder<'ctx>,

  pub var_map: HashMap<VariableReference, AnyValueEnum<'ctx>>,
}

fn parse_int_literal(text: &str) -> u64 {
  if text.starts_with(consts::punctuation::BIN_PFX) {
    todo!("parse_int_literal bin");
  };

  if text.starts_with(consts::punctuation::OCT_PFX) {
    todo!("parse_int_literal oct");
  };

  if text.starts_with(consts::punctuation::HEX_PFX) {
    todo!("parse_int_literal hex");
  };

  let clean = text.chars().filter(|ch| *ch != '_').collect::<String>();

  clean.parse()
    .expect("failed to parse int literal")
}

impl<'a, 'ctx> Codegen<'a, 'ctx> {
  pub fn new(context: &'ctx Context, module: &'a Module<'ctx>, builder: &'a Builder<'ctx>) -> Codegen<'a, 'ctx> {
    Self {
      context, module, builder,
      var_map: HashMap::new(),
    }
  }

  pub fn generate(&mut self, global: &NamespaceAST) -> CodeGenResult<()> {
    self.generate_namespace(global)?;

    if let Err(err) = self.module.verify() {
      return ValidationFailedSnafu {
        message: err.to_string()
      }.fail();
    };

    Ok(())
  }
}
