/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

mod errors;
use errors::*;

use crate::aster::ast::NamespaceAST;

// use inkwell::context::Context;
// use inkwell::builder::Builder;
// use inkwell::module::Module;

use inkwell::context::Context;
// use inkwell::values::{FloatValue, FunctionValue};
// use inkwell::FloatPredicate;
use inkwell::{
  builder::Builder,
  // values::BasicValueEnum
};
use inkwell::{
  module::Module,
  // values::PointerValue
};

pub struct Codegen<'a, 'ctx> {
  pub context: &'ctx Context,
  pub module: &'a Module<'ctx>,
  pub builder: &'a Builder<'ctx>,
}

impl Codegen<'_, '_> {
  // pub fn init(&mut self, filename: &str) {
  //   self.module.set_source_file_name(filename);
  //   self.generate_main_fn();
  //   self.add_printf();
  // }
}

impl Codegen<'_, '_> {
  pub fn generate_namespace(&mut self, ns: &NamespaceAST) -> CodeGenResult<()> {
    NotImplementedSnafu { what: "Code generation" }.fail()
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
