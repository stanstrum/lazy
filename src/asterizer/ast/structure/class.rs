use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError,
  TemplateScope,
  Block,
  Type,
};

use crate::tokenizer::{
  Grouping,
  GroupingType,
  Keyword,
  Operator,
  Punctuation,
  Span,
  GetSpan,
  TokenEnum,
};

use crate::asterizer::error::ExpectedSnafu;

#[allow(unused)]
#[derive(Debug)]
pub(crate) enum MemberVisibility {
  Private,
  Protected,
  Public,
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct Field {
  pub(crate) visibility: MemberVisibility,
  pub(crate) r#static: bool,
  pub(crate) r#mut: bool,
  pub(crate) name: String,
  pub(crate) ty: Type,
  pub(crate) span: Span,
}

#[derive(Debug)]
pub(crate) enum MethodKind {
  Static,
  Consume,
  MutConsume,
  Reference,
  MutReference,
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct MethodArgument {
  pub(crate) name: String,
  pub(crate) ty: Type,
  pub(crate) span: Span,
}

#[derive(Debug)]
pub(crate) enum MethodBody {
  Abstract,
  Implementation(Block),
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct MethodArguments {
  pub(crate) kind: MethodKind,
  pub(crate) args: Option<Vec<MethodArgument>>,
  pub(crate) span: Span,
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct Method {
  pub(crate) visibility: MemberVisibility,
  pub(crate) name: String,
  pub(crate) return_ty: Option<Type>,
  pub(crate) args: MethodArguments,
  pub(crate) body: MethodBody,
  pub(crate) span: Span,
}

#[derive(Debug, TypeName)]
pub(crate) enum ClassMember {
  Field(Field),
  Method(Method),
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct ClassChild {
  pub(crate) template: Option<TemplateScope>,
  pub(crate) body: ClassMember,
  pub(crate) span: Span,
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct Class {
  pub(crate) name: String,
  pub(crate) children: Vec<ClassChild>,
  pub(crate) span: Span,
}

impl GetSpan for MemberVisibility {
  fn get_span(&self) -> Span {
    todo!()
  }
}

impl GetSpan for Field {
  fn get_span(&self) -> Span {
    self.span
  }
}

impl GetSpan for MethodKind {
  fn get_span(&self) -> Span {
    todo!()
  }
}

impl GetSpan for MethodArgument {
  fn get_span(&self) -> Span {
    self.span
  }
}

impl GetSpan for MethodBody {
  fn get_span(&self) -> Span {
    todo!()
  }
}

impl GetSpan for MethodArguments {
  fn get_span(&self) -> Span {
    self.span
  }
}

impl GetSpan for Method {
  fn get_span(&self) -> Span {
    self.span
  }
}

impl GetSpan for ClassMember {
  fn get_span(&self) -> Span {
    match self {
      ClassMember::Field(field) => field.get_span(),
      ClassMember::Method(method) => method.get_span(),
    }
  }
}

impl GetSpan for ClassChild {
  fn get_span(&self) -> Span {
    self.span
  }
}

impl GetSpan for Class {
  fn get_span(&self) -> Span {
    self.span
  }
}

impl MakeAst for Field {
  fn make(_stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    // TODO: implement this
    Ok(None)
  }
}

impl MakeAst for MethodArgument {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(TokenEnum::Identifier(name)) = stream.next_variant() else {
      return Ok(None);
    };
    let name = name.to_owned();

    stream.skip_whitespace_and_comments();

    let Some(TokenEnum::Punctuation(Punctuation::Colon)) = stream.next_variant() else {
      return ExpectedSnafu {
        what: "a colon",
        span: stream.span(),
      }.fail();
    };

    stream.skip_whitespace_and_comments();

    let Some(ty) = stream.make()? else {
      return ExpectedSnafu {
        what: "a type",
        span: stream.span(),
      }.fail();
    };

    Ok(Some(Self {
      name,
      ty,
      span: stream.span_mark(),
    }))
  }
}

impl MakeAst for MethodArguments {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let mut kind = MethodKind::Static;
    let mut args = None;

    if let Some(TokenEnum::Punctuation(Punctuation::Colon)) = stream.peek_variant() {
      stream.seek();
      stream.skip_whitespace_and_comments();

      let mut read_reference = false;
      let mut read_mut = false;

      loop {
        match stream.peek_variant() {
          Some(TokenEnum::Operator(Operator::SingleAnd)) if !read_reference && !read_mut => {
            stream.seek();
            stream.skip_whitespace_and_comments();

            read_reference = true;
          },
          Some(TokenEnum::Keyword(Keyword::Mut)) if !read_mut => {
            stream.seek();
            stream.skip_whitespace_and_comments();

            read_mut = true;
          },
          Some(TokenEnum::Identifier(ident)) if ident == "this" => {
            stream.seek();

            kind = match (read_reference, read_mut) {
              (false, false) => MethodKind::Consume,
              (false, true) => MethodKind::MutConsume,
              (true, false) => MethodKind::Reference,
              (true, true) => MethodKind::MutReference,
            };

            break;
          },
          _ => break,
        };
      };

      'args: {
        stream.push_mark();
        stream.skip_whitespace_and_comments();

        if !matches!(kind, MethodKind::Static) {
          let Some(TokenEnum::Punctuation(Punctuation::Comma)) = stream.next_variant() else {
            stream.pop_mark();

            break 'args;
          };
        };

        stream.drop_mark();
        args = Some(vec![]);

        loop {
          stream.skip_whitespace_and_comments();

          let Some(arg) = stream.make()? else {
            return ExpectedSnafu {
              what: "a method argument",
              span: stream.span(),
            }.fail();
          };

          args.as_mut().unwrap().push(arg);

          stream.push_mark();
          stream.skip_whitespace_and_comments();

          let Some(TokenEnum::Punctuation(Punctuation::Comma)) = stream.next_variant() else {
            stream.pop_mark();

            break;
          };

          stream.drop_mark();
        };
      };
    };

    Ok(Some(Self {
      kind,
      args,
      span: stream.span_mark(),
    }))
  }
}

impl MakeAst for Method {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let visibility = match stream.peek_variant() {
      Some(TokenEnum::Keyword(Keyword::Private)) => Some(MemberVisibility::Private),
      Some(TokenEnum::Keyword(Keyword::Protected)) => Some(MemberVisibility::Protected),
      _ => None
    };

    if visibility.is_some() {
      stream.seek();
      stream.skip_whitespace_and_comments();
    };

    let visibility = visibility.unwrap_or(MemberVisibility::Public);

    let r#abstract = {
      if let Some(TokenEnum::Keyword(Keyword::Abstract)) = stream.peek_variant() {
        stream.seek();
        stream.skip_whitespace_and_comments();

        true
      } else {
        false
      }
    };

    let Some(TokenEnum::Identifier(name)) = stream.next_variant() else {
      return ExpectedSnafu {
        what: "an identifier",
        span: stream.span(),
      }.fail();
    };
    let name = name.to_owned();

    stream.skip_whitespace_and_comments();

    let return_ty = {
      if let Some(TokenEnum::Operator(Operator::RightArrow)) = stream.peek_variant() {
        stream.seek();
        stream.skip_whitespace_and_comments();

        let Some(ty) = stream.make()? else {
          return ExpectedSnafu {
            what: "a type",
            span: stream.span(),
          }.fail();
        };

        stream.skip_whitespace_and_comments();

        Some(ty)
      } else {
        None
      }
    };

    let args = stream.make()?.expect("method arguments failed");

    let body = if !r#abstract {
      stream.skip_whitespace_and_comments();

      let Some(block) = stream.make()? else {
        return ExpectedSnafu {
          what: "a block expression",
          span: stream.span(),
        }.fail();
      };

      MethodBody::Implementation(block)
    } else {
      MethodBody::Abstract
    };

    Ok(Some(Self {
      visibility,
      name,
      return_ty,
      args,
      body,
      span: stream.span_mark(),
    }))
  }
}

impl MakeAst for ClassChild {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let template = stream.make()?;

    if template.is_some() {
      stream.skip_whitespace_and_comments();
    };

    #[allow(clippy::manual_map)]
    Ok({
      if let Some(field) = stream.make()? {
        Some(Self {
          template,
          body: ClassMember::Field(field),
          span: stream.span_mark(),
        })
      } else if let Some(method) = stream.make()? {
        Some(Self {
          template,
          body: ClassMember::Method(method),
          span: stream.span_mark(),
        })
      } else {
        None
      }
    })
  }
}

impl MakeAst for Class {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(TokenEnum::Keyword(Keyword::Class)) = stream.next_variant() else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let Some(TokenEnum::Identifier(name)) = stream.next_variant() else {
      return ExpectedSnafu {
        what: "an identifier",
        span: stream.span(),
      }.fail();
    };
    let name = name.to_owned();

    stream.skip_whitespace_and_comments();

    // TODO: implements & extends here

    let Some(TokenEnum::Grouping(Grouping::Open(GroupingType::CurlyBrace))) = stream.next_variant() else {
      return ExpectedSnafu {
        what: "an opening curly brace",
        span: stream.span(),
      }.fail();
    };

    let mut children = vec![];
    loop {
      stream.skip_whitespace_and_comments();

      if let Some(TokenEnum::Grouping(Grouping::Close(GroupingType::CurlyBrace))) = stream.peek_variant() {
        stream.seek();

        break;
      };

      let Some(child) = stream.make()? else {
        return ExpectedSnafu {
          what: "a class member",
          span: stream.span(),
        }.fail();
      };

      children.push(child);
      stream.skip_whitespace_and_comments();

      let Some(TokenEnum::Punctuation(Punctuation::Semicolon)) = stream.next_variant() else {
        return ExpectedSnafu {
          what: "a semicolon",
          span: stream.span(),
        }.fail();
      };
    };

    Ok(Some(Self {
      name,
      children,
      span: stream.span_mark(),
    }))
  }
}
