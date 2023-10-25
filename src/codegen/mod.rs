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
mod binary;

use std::collections::HashMap;

use errors::*;
pub use errors::CodeGenError;

use crate::aster::{
  ast::*,
  consts
};

use inkwell::{
  module::Module,
  context::Context,
  builder::Builder,
  values::AnyValueEnum,
};

pub struct Codegen<'a, 'ctx> {
  pub context: &'ctx Context,
  pub module: &'a Module<'ctx>,
  pub builder: &'a Builder<'ctx>,

  pub var_map: HashMap<*const BindingAST, AnyValueEnum<'ctx>>,
  pub extern_map: HashMap<*const ExternDeclAST, AnyValueEnum<'ctx>>,
  pub func_map: HashMap<*const FunctionAST, AnyValueEnum<'ctx>>,
  // pub arg_map: HashMap<*const BindingAST, AnyValueEnum<'ctx>>,
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
      extern_map: HashMap::new(),
      func_map: HashMap::new()
    }
  }

  pub fn get_var_ref<'b>(&'b self, var_ref: &VariableReference) -> Option<&'b AnyValueEnum<'ctx>> {
    match var_ref {
      VariableReference::ResolvedVariable(ptr) => {
        self.var_map.get(ptr)
      },
      VariableReference::ResolvedArgument(_) => todo!(),
      VariableReference::ResolvedFunction(ptr) => {
        self.func_map.get(ptr)
      },
      VariableReference::ResolvedMemberFunction(_) => todo!(),
      VariableReference::ResolvedMemberOf(..) => todo!(),
      VariableReference::ResolvedExternal(ptr) => {
        self.extern_map.get(ptr)
      },
    }
  }

  pub fn insert_var_ref(&mut self, var_ref: VariableReference, value: AnyValueEnum<'ctx>) -> Option<AnyValueEnum<'ctx>> {
    match var_ref {
      VariableReference::ResolvedVariable(ptr) => {
        self.var_map.insert(ptr, value)
      },
      VariableReference::ResolvedArgument(_) => todo!(),
      VariableReference::ResolvedFunction(ptr) => {
        self.func_map.insert(ptr, value)
      },
      VariableReference::ResolvedMemberFunction(_) => todo!(),
      VariableReference::ResolvedMemberOf(..) => todo!(),
      VariableReference::ResolvedExternal(ptr) => {
        self.extern_map.insert(ptr, value)
      },
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
