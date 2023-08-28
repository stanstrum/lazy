/* Copyright (c) 2023, Stan Strum
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree.
 */

 use super::super::{
  super::{
    ast::*,
    SourceReader,
    AsterResult,
    seek_read::seek,
    consts,
    errors::*,
  },
  try_make,
};

impl AtomExpressionAST {
  fn make_blind_binding(reader: &mut SourceReader, ty: Option<TypeAST>) -> AsterResult<Self> {
    let start = reader.offset();

    let ident = IdentAST::make(reader)?;

    seek::optional_whitespace(reader)?;

    if !seek::begins_with(reader, consts::punctuation::BOLLOCKS) {
      return ExpectedSnafu {
        what: "Punctuation (:=)",
        offset: reader.offset()
      }.fail();
    };

    seek::optional_whitespace(reader)?;

    let value = Box::new(Expression::make(reader)?);

    let out = match ty {
      Some(TypeAST { ref e, .. }) => e.clone(),
      _ => Type::Unresolved,
    };

    Ok(
      Self {
        a: AtomExpression::Binding {
          ident, ty, value
        },
        span: reader.span_since(start),
        out
      }
    )
  }

  pub fn make_binding(reader: &mut SourceReader) -> AsterResult<Self> {
    if let Some(binding) = try_make!(Self::make_blind_binding, reader, None) {
      return Ok(binding);
    };

    let ty = try_make!(TypeAST::make, reader);

    if ty.is_some() {
      seek::required_whitespace(reader)?;
    };

    if let Some(binding) = try_make!(Self::make_blind_binding, reader, ty) {
      Ok(binding)
    } else {
      ExpectedSnafu {
        what: "Binding",
        offset: reader.offset()
      }.fail()
    }
  }

  fn make_fn_call(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    let callee = if let Some(qual) = try_make!(QualifiedAST::make, reader) {
      FnCallee::Qualified(qual)
    } else if let Some(sub_expr) = try_make!(SubExpressionAST::make, reader) {
      FnCallee::SubExpression(sub_expr)
    } else {
      return ExpectedSnafu {
        what: "Ident or Sub-Expression",
        offset: reader.offset()
      }.fail();
    };

    seek::optional_whitespace(reader)?;

    if !seek::begins_with(reader, consts::grouping::OPEN_PARENTHESIS) {
      return ExpectedSnafu {
        what: "Open Parenthesis",
        offset: reader.offset()
      }.fail();
    };

    let mut args: Vec<Expression> = vec![];

    loop {
      seek::optional_whitespace(reader)?;

      if seek::begins_with(reader, consts::grouping::CLOSE_PARENTHESIS) {
        break;
      };

      let arg_expr = Expression::make(reader)?;
      args.push(arg_expr);

      seek::optional_whitespace(reader)?;

      if !seek::begins_with(reader, consts::punctuation::COMMA) {
        if !seek::begins_with(reader, consts::grouping::CLOSE_PARENTHESIS) {
          return ExpectedSnafu {
            what: "Close Parenthesis",
            offset: reader.offset()
          }.fail();
        } else {
          break;
        };
      };
    };

    Ok(Self {
      span: reader.span_since(start),
      out: Type::Unresolved,
      a: AtomExpression::FnCall(Box::new(callee), args)
    })
  }

  pub fn make(reader: &mut SourceReader) -> AsterResult<Self> {
    let start = reader.offset();

    let a = if let Some(assn) = try_make!(AtomExpressionAST::make_binding, reader) {
      assn
    } else if let Some(lit) = try_make!(LiteralAST::make, reader) {
      Self {
        span: reader.span_since(start),
        a: AtomExpression::Literal(lit),
        out: Type::Unresolved,
      }
    } else if let Some(fn_call) = try_make!(AtomExpressionAST::make_fn_call, reader) {
      fn_call
    } else if let Some(qual) = try_make!(QualifiedAST::make, reader) {
      Self {
        span: reader.span_since(start),
        out: Type::Unresolved,
        a: AtomExpression::Variable(qual)
      }
    } else {
      return UnknownSnafu {
        what: "Expression",
        offset: reader.offset()
      }.fail();
    };

    seek::optional_whitespace(reader)?;

    if let Some(b) = {
      if seek::begins_with(reader, consts::operator::ADD) {
        seek::optional_whitespace(reader)?;
        try_make!(Expression::make, reader)
      } else {
        None
      }
    } {
      Ok(Self {
        span: reader.span_since(start),
        out: Type::Unresolved,
        a: AtomExpression::OperatorExpr(OperatorExpr::Add(
          Box::new(Expression::Atom(a)),
          Box::new(b)
        ))
      })
    } else if let Some(b) = {
      if seek::begins_with(reader, consts::operator::SUB) {
        seek::optional_whitespace(reader)?;
        try_make!(Expression::make, reader)
      } else {
        None
      }
    } {
      Ok(Self {
        span: reader.span_since(start),
        out: Type::Unresolved,
        a: AtomExpression::OperatorExpr(OperatorExpr::Sub(
          Box::new(Expression::Atom(a)),
          Box::new(b)
        ))
      })
    } else if let Some(b) = {
      if seek::begins_with(reader, consts::operator::MUL) {
        seek::optional_whitespace(reader)?;
        try_make!(Expression::make, reader)
      } else {
        None
      }
    } {
      Ok(Self {
        span: reader.span_since(start),
        out: Type::Unresolved,
        a: AtomExpression::OperatorExpr(OperatorExpr::Mul(
          Box::new(Expression::Atom(a)),
          Box::new(b)
        ))
      })
    } else if let Some(b) = {
      if seek::begins_with(reader, consts::operator::DIV) {
        seek::optional_whitespace(reader)?;
        try_make!(Expression::make, reader)
      } else {
        None
      }
    } {
      Ok(Self {
        span: reader.span_since(start),
        out: Type::Unresolved,
        a: AtomExpression::OperatorExpr(OperatorExpr::Div(
          Box::new(Expression::Atom(a)),
          Box::new(b)
        ))
      })
    } else if let Some(b) = {
      if seek::begins_with(reader, consts::operator::MOD) {
        seek::optional_whitespace(reader)?;
        try_make!(Expression::make, reader)
      } else {
        None
      }
    } {
      Ok(Self {
        span: reader.span_since(start),
        out: Type::Unresolved,
        a: AtomExpression::OperatorExpr(OperatorExpr::Mod(
          Box::new(Expression::Atom(a)),
          Box::new(b)
        ))
      })
    } else {
      Ok(a)
    }
  }
}
