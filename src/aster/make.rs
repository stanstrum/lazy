/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::aster::to_string::str_line_pfx;

use super::intrinsics;

use super::{
  ast::*, errors::*,
  source_reader::SourceReader,
  seek_read::seek,
  consts
};

impl IdentAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    let mut text = String::new();

    for ctr in 0usize.. {
      let Ok(first) = reader.read(1) else {
        return ExpectedSnafu { what: "More characters (for Ident)", offset: reader.offset() }.fail();
      };

      let first = first.chars().nth(0).unwrap();

      match first {
        '_' => {},
        _ if first.is_alphabetic() => {},
        _ if ctr != 0 && first.is_numeric() => {},
        _ if ctr == 0 => {
          reader.rewind(1).unwrap();

          return ExpectedSnafu { what: "Ident", offset: reader.offset() }.fail();
        },
        _ => {
          reader.rewind(1).unwrap();

          break;
        }
      };

      text.push(first);
    };

    let span = reader.span_since(start);

    Ok(Self { span, text })
  }
}

impl TypeAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    println!("{}", reader.at());

    if seek::begins_with(reader, consts::punctuation::AMPERSAND) {
      println!("TypeAST::make ReferenceTo:");

      // if seek::begins_with(reader, consts::keyword::MUT) {
      //   todo!();
      // };

      let ty = Box::new(TypeAST::make(reader)?);

      Ok(Self {
        span: reader.span_since(start),
        e: Type::ConstReferenceTo(ty)
      })
    } else if seek::begins_with(reader, consts::grouping::OPEN_BRACKET) {
      let mut len: Option<u32> = None;

      loop {
        let ch = reader.read_ch();

        match ch {
          Ok('0'..='9') => {
            len = Some(
              // sponge: this is bad
              ch.unwrap().to_digit(10).unwrap() +
              len.unwrap_or_default()
            );
          },
          Ok(']') => {
            break;
          },
          _ => todo!("bad array size due to naive parsing")
        }
      };

      let ty = Box::new(TypeAST::make(reader)?);

      Ok(Self {
        span: reader.span_since(start),
        e: Type::ArrayOf(len, ty)
      })
    } else if let Some(e) = 'ident: {
      let Some(ident) = try_make(IdentAST::make, reader, "IdentAST::make for TypeAST::make") else {
        break 'ident None;
      };

      // println!("TypeAST{}", ident.to_string());

      intrinsics::get_intrinsic(&ident.text)
     } {
      println!("TypeAST::make get_intrinsic:");

      Ok(Self {
        span: reader.span_since(start),
        e
      })
    } else {
      println!("TypeAST::make fail:");

      UnknownSnafu { what: "Type", offset: reader.offset() }.fail()
    }
  }
}

fn try_make<T>(mut f: impl FnMut(&mut SourceReader) -> AsterResult<T>, reader: &mut SourceReader, pfx: &str) -> Option<T> {
  let start = reader.offset();
  let res = f(reader);

  match res {
    Ok(v) => Some(v),
    Err(e) => {
      println!(
        "{}",
        str_line_pfx(
          format!(
            "{} at:\n{}",
            e.to_string(),
            reader.at()
          ),
          format!("{}: ", pfx).as_str()
        )
      );

      reader.rewind(reader.offset() - start).unwrap();

      None
    },
  }
}

impl Expression {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    if let Some(expr) = try_make(BlockExpressionAST::make, reader, "BlockExpressionAST::make for Expression::make") {
      Ok(Expression::Block(expr))
    } else if let Some(expr) = try_make(AtomExpressionAST::make, reader, "AtomExpressionAST::make for Expression::make") {
      Ok(Expression::Atom(expr))
    } else {
      ExpectedSnafu { what: "Expression (BlockExpression, AtomExpression)", offset: reader.offset() }.fail()
    }
  }
}

impl AtomExpressionAST {
  pub fn make_literal(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    if seek::begins_with(reader, consts::punctuation::QUOTE) {
      let mut text = String::new();

      loop {
        let Ok(ch) = reader.read(1) else {
          return ExpectedSnafu { what: "Closing quote (\")", offset: reader.offset() }.fail();
        };

        match ch {
          consts::punctuation::QUOTE => {
            break;
          },
          consts::punctuation::BACKSLASH => {
            match reader.read_ch() {
              Ok('\'') => text.push('\''),
              Ok('"') => text.push('"'),
              Ok('\\') => text.push('\\'),
              Ok(consts::ascii::NL_ESCAPE) => text.push(consts::ascii::NL),
              Ok(consts::ascii::BL_ESCAPE) => text.push(consts::ascii::BL),
              Ok(consts::ascii::BS_ESCAPE) => text.push(consts::ascii::BS),
              Ok(consts::ascii::HT_ESCAPE) => text.push(consts::ascii::HT),
              Ok(consts::ascii::LF_ESCAPE) => text.push(consts::ascii::LF),
              Ok(consts::ascii::VT_ESCAPE) => text.push(consts::ascii::VT),
              Ok(consts::ascii::FF_ESCAPE) => text.push(consts::ascii::FF),
              Ok(consts::ascii::CR_ESCAPE) => text.push(consts::ascii::CR),
              Ok(consts::ascii::ES_ESCAPE) => text.push(consts::ascii::ES),
              Ok(consts::ascii::HEX_ESCAPE) => {
                return NotImplementedSnafu {
                  what: "Hexadecimal Escape",
                  offset: reader.offset()
                }.fail();
              },
              Ok(consts::ascii::UNI_ESCAPE) => {
                return NotImplementedSnafu {
                  what: "Unicode Escape",
                  offset: reader.offset()
                }.fail();
              },
              Ok(escaped) => {
                return UnknownSnafu {
                  what: format!(
                    "Escaped Character ({:#?})",
                    escaped
                  ),
                  offset:
                  reader.offset()
                }.fail();
              },
              Err(_) => {
                return ExpectedSnafu {
                  what: "Escaped Character",
                  offset: reader.offset()
                }.fail();
              }
            }
          },
          _ => {
            text.push_str(ch);
          }
        };
      };

      Ok(Self {
        span: reader.span_since(start),
        a: AtomExpression::Literal(
          Literal::String(text)
        ),
        out: Type::Unresolved
      })
    } else {
      ExpectedSnafu { what: "Literal", offset: reader.offset() }.fail()
    }
  }

  pub fn make_assignment(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    let ty = try_make(TypeAST::make, reader, "TypeAST::make for AtomicExpressionAST::make_assignment");

    match ty {
      Some(_) => { seek::required_whitespace(reader)?; },
      _ => {}
    };

    let ident = IdentAST::make(reader)?;

    seek::optional_whitespace(reader)?;

    if !seek::begins_with(reader, consts::punctuation::BOLLOCKS) {
      return ExpectedSnafu { what: "Punctuation (:=)", offset: reader.offset() }.fail();
    };

    seek::optional_whitespace(reader)?;

    let value = Box::new(Expression::make(reader)?);

    let out = match ty {
      Some(TypeAST { ref e, .. }) => e.clone(),
      _ => Type::Unresolved,
    };

    Ok(
      Self {
        a: AtomExpression::Assignment {
          ident, ty, value
        },
        span: reader.span_since(start),
        out
      }
    )
  }

  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    // assignment

    if let Some(assn) = try_make(Self::make_assignment, reader, "AtomExpressionAST::make_assignment for AtomicExpressionAST::make") {
      Ok(assn)
    } else if let Some(lit) = try_make(Self::make_literal, reader, "AtomExpressionAST::make_literal for AtomicExpressionAST::make") {
      Ok(lit)
    } else {
      UnknownSnafu { what: "Expression", offset: reader.offset() }.fail()
    }
  }
}

impl BlockExpressionAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    if !seek::begins_with(reader, consts::grouping::OPEN_BRACE) {
      return ExpectedSnafu { what: "Open Block Expression ({)", offset: reader.offset() }.fail();
    };

    let mut children: Vec<Expression> = vec![];

    let returns_last = loop {
      seek::optional_whitespace(reader)?;

      if seek::begins_with(reader, consts::grouping::CLOSE_BRACE) {
        break false;
      };

      if let Ok(expr) = AtomExpressionAST::make(reader) {
        children.push(Expression::Atom(expr));
      } else if let Ok(expr) = BlockExpressionAST::make(reader) {
        children.push(Expression::Block(expr));
      } else {
        return ExpectedSnafu { what: "Expression (block, atom)", offset: reader.offset() }.fail();
      };

      seek::optional_whitespace(reader)?;

      if !seek::begins_with(reader, consts::punctuation::SEMICOLON) {
        if seek::begins_with(reader, consts::grouping::CLOSE_BRACE) {
          break true;
        } else {
          return ExpectedSnafu { what: "Close Block Expression (}) or Semicolon", offset: reader.offset() }.fail();
        };
      };
    };

    Ok(Self {
      span: reader.span_since(start),
      children, returns_last, out: Type::Unresolved
    })
  }
}

impl FunctionAST {
  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    // "fn"
    if !seek::begins_with(reader, consts::keyword::FN) {
      return ExpectedSnafu { what: "Keyword (fn)", offset: reader.offset() }.fail();
    };

    seek::required_whitespace(reader)?;

    let ident = IdentAST::make(reader)?;

    seek::optional_whitespace(reader)?;

    // return type (optional)
    let ret = if seek::begins_with(reader, consts::punctuation::RIGHT_ARROW) {
      seek::optional_whitespace(reader)?;

      let ret = TypeAST::make(reader)?;

      seek::optional_whitespace(reader)?;

      ret
    } else {
      // sponge: this will show a type error with a void return type in a nontrivial place
      // due to the fact that a void return type is implicitly inferred
      TypeAST {
        span: reader.span_since(start),
        e: intrinsics::get_intrinsic("void")
          .expect("Could not resolve `void` intrinsic")
      }
    };

    let mut args: Vec<Variable> = vec![];

    // arguments (optional)
    if seek::begins_with(reader, consts::punctuation::COLON) {
      loop {
        seek::optional_whitespace(reader)?;

        let arg_ty = TypeAST::make(reader)?;

        seek::required_whitespace(reader)?;

        let arg_ident = IdentAST::make(reader)?;

        seek::optional_whitespace(reader)?;

        args.push(Variable(arg_ty, arg_ident));

        if !seek::begins_with(reader, consts::punctuation::COMMA) {
          break;
        }
      }

      seek::optional_whitespace(reader)?;
    };

    let body = BlockExpressionAST::make(reader)?;

    Ok(Self {
      span: reader.span_since(start),
      ident, args, ret, body
    })
  }
}

mod tests {
  use crate::aster::{asterize, source_reader::formatting::Message};

  #[allow(unused)]
  use super::*;

  macro_rules! snippet_test {
    ($name:ident, $reader:ident => $body:tt) => {
      #[test]
      fn $name() {
        let src_as_str = include_str!(concat!("../snippets/", stringify!($name), ".zy"));
        let src = src_as_str.to_string();

        let ref mut $reader = SourceReader::new(
          concat!("../snippets/", stringify!($name), ".zy").to_string(),
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
    show_message, reader => {
      let global = asterize(reader).unwrap();

      dbg!(&global);

      let main = global.map.get("main").unwrap();
      let main = match main {
        Structure::NamespaceAST(_) => panic!("main is of wrong structure type"),
        Structure::FunctionAST(main) => main,
      };

      let expr = main.body.children.get(0).unwrap();

      let mes = Message {
        level: crate::aster::source_reader::formatting::Level::Debug,
        msg: "testing 1234".to_string(),
        span: expr.span(),
      };

      println!("{}", reader.show_message(mes));
    }
  );
}
