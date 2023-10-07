/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::aster::ast::{
  QualifiedAST,
  IdentAST
};

use std::io::Write;

impl std::string::ToString for QualifiedAST {
  fn to_string(&self) -> String {
    let mut w: Vec<u8> = vec![];

    // will always have at least 1
    let last = self.parts.len() - 1;

    for (i, part) in self.parts.iter().enumerate() {
      write!(&mut w, "{}", part.to_string()).unwrap();

      if i != last {
        write!(&mut w, "::").unwrap();
      };
    };

    String::from_utf8(w).unwrap()
  }
}

impl std::string::ToString for IdentAST {
  fn to_string(&self) -> String {
    self.text.to_owned()
  }
}
