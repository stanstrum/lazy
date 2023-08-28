/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

mod r#type;
mod ident;
mod expression;
mod qualified;
mod literal;
mod function;
mod structure;
mod r#trait;
mod member_function;
mod keyword;
mod r#impl;

pub use super::{
  seek_read::{seek, read},
  ast::*,
  formatting::*
};

#[macro_export]
macro_rules! try_make {
  ($func:expr, $reader:ident $(, $args:expr)*) => {{
    // use crate::aster::formatting::*;

    let start = $reader.offset();
    // let text = concat!(stringify!($func), " from ", file!(), ":", line!());

    // println!("{}", format_message($reader.src(), Message {
    //   level: Level::Note,
    //   msg: format!("Trying {}", text),
    //   sub: "here".to_owned(),
    //   span: Span {
    //     start: $reader.offset(),
    //     end: $reader.offset()
    //   }
    // }));

    let res = $func($reader $(, $args)*);

    match res {
      Ok(v) => {
        // let msg = Message {
        //   level: Level::Debug,
        //   msg: format!("Successfully parsed {}", text),
        //   sub: "here".to_owned(),
        //   span: v.span()
        // };

        // println!("{}", format_message($reader.src(), msg));

        Some(v)
      },
      Err(e) => {
        // let message = Message {
        //   level: Level::Warning,
        //   msg: format!("Failed to parse {}", text),
        //   sub: e.to_string(),
        //   span: Span {
        //     start, end: start
        //   }
        // };

        // println!("{}", format_message($reader.src(), message));

        $reader.rewind($reader.offset() - start).unwrap();

        None
      },
    }
  }};
}

pub use try_make;

#[allow(unused_imports)]
mod tests {
  use crate::aster::SourceReader;
  use crate::aster::ast::*;
  use crate::aster::seek_read::seek;
  use crate::aster::asterize;
  use crate::aster::formatting::*;

  macro_rules! snippet_test {
    ($name:ident, $reader:ident => $body:tt) => {
      #[test]
      fn $name() {
        let src_as_str = include_str!(concat!("../../snippets/", stringify!($name), ".zy"));
        let src = src_as_str.to_string();

        let ref mut $reader = SourceReader::new(
          concat!("../../snippets/", stringify!($name), ".zy").to_string(),
          &src
        );

        $body
      }
    };
  }

  snippet_test!(
    type_make, reader => {
      let b = TypeAST::make(reader)
        .unwrap();

      println!("{:#?}", b);

      reader.read_ch().unwrap();

      seek::optional_whitespace(reader).unwrap();
      assert!(reader.remaining() == 0);
    }
  );

  snippet_test!(
    message, reader => {
      let global = asterize(reader).unwrap();

      dbg!(&global);

      let main = global.map.get("main").unwrap();
      let main = match main {
        Structure::Function(main) => main,
        _ => panic!("main is of wrong structure type"),
      };

      let expr = main.body.children.get(0).unwrap();

      let mes = Message {
        level: crate::aster::source_reader::formatting::Level::Debug,
        msg: "testing 1234".to_owned(),
        sub: "sub message".to_owned(),
        span: expr.span(),
      };

      println!("{}", format_message(reader.src(), mes));
    }
  );

  snippet_test!(
    string_ref, reader => {
      let assn = AtomExpressionAST::make_binding(reader);

      dbg!(&assn);

      seek::optional_whitespace(reader).unwrap();
      assert!(reader.remaining() == 0);
    }
  );
}
