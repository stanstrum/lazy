/* Copyright (c) 2024 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::asterizer::ast::GlobalNamespace;
use crate::tokenizer::Token;

pub(crate) fn tokens(toks: &Vec<Token>) {
  dbg!(toks);

  let source = toks.iter()
    .map(std::string::ToString::to_string)
    .reduce(|acc, e| acc + &e)
    .expect("failed to accumulate source code");

  println!("{source}");
}

pub(crate) fn ast(ast: &GlobalNamespace) {
  dbg!(&ast);
}
