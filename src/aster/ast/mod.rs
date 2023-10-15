/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

mod structure;
mod ident;
mod expression;
mod operator;
mod r#type;

use std::path::PathBuf;

pub use structure::*;
pub use ident::*;
pub use expression::*;
pub use operator::*;
pub use r#type::*;

#[derive(Debug, Clone)]
pub struct Span {
  pub start: usize,
  pub end: usize,
  pub path: PathBuf
}

pub trait GetSpan {
  fn span(&self) -> Span;
}

#[macro_export]
macro_rules! make_get_span [
  ($i:ident) => {
    impl GetSpan for $i {
      fn span(&self) -> Span {
        self.span.clone()
      }
    }
  };

  ($first:ident, $($rest:ident),+) => {
    make_get_span!($first);
    make_get_span!($($rest),+);
  };
];
