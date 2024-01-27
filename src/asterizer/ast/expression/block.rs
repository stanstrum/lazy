/* Copyright (c) 2024 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use super::Expression;

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct BlockExpression {
  pub children: Vec<Expression>
}
