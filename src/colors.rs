/* Copyright (c) 2023 Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

macro_rules! def_colors {
  ($n:ident: $c:expr, $($ns:ident: $cs:expr),+) => {
    def_colors! { $n: $c }
    def_colors! { $($ns: $cs),+ }
  };

  ($n:ident: $c:expr) => {
    #[allow(unused)]
    pub const $n: &str = concat! { "\x1b[", $c, "m" };
  };
}

def_colors! {
  BLACK: "30",
  RED: "31",
  GREEN: "32",
  YELLOW: "33",
  BLUE: "34",
  MAGENTA: "35",
  CYAN: "36",
  LIGHT_GRAY: "37",
  DARK_GRAY: "90",
  LIGHT_RED: "91",
  LIGHT_GREEN: "92",
  LIGHT_YELLOW: "38;5;215",
  LIGHT_BLUE: "38;5;159",
  LIGHT_MAGENTA: "95",
  LIGHT_CYAN: "96",
  WHITE: "97",

  CREME: "38;5;230",
  TEAL: "38;5;159",
  MINT: "38;5;158",

  UNDERLINE: "4",
  BOLD: "1",
  CLEAR: "0"
}
