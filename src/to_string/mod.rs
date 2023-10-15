/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

mod ident;
mod structure;
mod r#type;
mod expression;
mod operator;

const INDENTATION: &str = "  ";

pub fn str_line_pfx(string: String, pfx: &str) -> String {
  let mut new_string = String::new();

  for line in string.split('\n') {
    if !new_string.is_empty() {
      new_string.push('\n');
    };

    if line.is_empty() {
      continue;
    };

    new_string.push_str(pfx);
    new_string.push_str(line);
  };

  new_string.trim_end().into()
}
