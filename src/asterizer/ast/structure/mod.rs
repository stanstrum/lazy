import_export!(namespace);
import_export!(type_alias);
import_export!(r#interface);
import_export!(r#struct);
import_export!(class);
import_export!(r#impl);
import_export!(r#extern);
import_export!(import);

use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError,
  Function,
  Type,
};

use crate::tokenizer::{
  Keyword,
  Punctuation,
  Span,
  GetSpan,
  TokenEnum,
};

use crate::asterizer::error::ExpectedSnafu;

#[derive(Debug, TypeName)]
pub(crate) enum TemplatableStructure {
  Function(Function),
  TypeAlias(TypeAlias),
  Interface(Interface),
  Struct(Struct),
  Class(Class),
  Exported(Exported),
  Impl(Impl),
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct UnconstrainedTemplateConstraint {
  pub(crate) name: String,
  pub(crate) span: Span,
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct ConstrainedTemplateConstraint {
  pub(crate) ty: Type,
  pub(crate) extends: Type,
  pub(crate) span: Span,
}

#[derive(Debug, TypeName)]
pub(crate) enum TemplateConstraint {
  Unconstrained(UnconstrainedTemplateConstraint),
  Extends(ConstrainedTemplateConstraint)
}

#[derive(Debug, TypeName)]
pub(crate) enum Structure {
  Namespace(Namespace),
  Function(Function),
  TypeAlias(TypeAlias),
  Interface(Interface),
  Struct(Struct),
  Class(Class),
  Impl(Impl),
  Extern(Extern),
  Import(Import),
  Exported(Exported),
  TemplateScope(TemplateScope),
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct TemplateScope {
  pub(crate) constraints: Vec<TemplateConstraint>,
  pub(crate) structure: TemplatableStructure,
  pub(crate) span: Span,
}

#[allow(unused)]
#[derive(Debug, TypeName)]
pub(crate) struct Exported {
  pub(crate) structure: Box<Structure>,
  pub(crate) span: Span,
}

impl GetSpan for TemplatableStructure {
  fn get_span(&self) -> Span {
    match self {
      TemplatableStructure::Function(function) => function.get_span(),
      TemplatableStructure::TypeAlias(typealias) => typealias.get_span(),
      TemplatableStructure::Interface(interface) => interface.get_span(),
      TemplatableStructure::Struct(r#struct) => r#struct.get_span(),
      TemplatableStructure::Class(class) => class.get_span(),
      TemplatableStructure::Exported(exported) => exported.get_span(),
      TemplatableStructure::Impl(r#impl) => r#impl.get_span(),
    }
  }
}

impl GetSpan for UnconstrainedTemplateConstraint {
  fn get_span(&self) -> Span {
    self.span
  }
}

impl GetSpan for ConstrainedTemplateConstraint {
  fn get_span(&self) -> Span {
    self.span
  }
}

impl GetSpan for TemplateConstraint {
  fn get_span(&self) -> Span {
    todo!()
  }
}

impl GetSpan for Structure {
  fn get_span(&self) -> Span {
    match self {
      Structure::Namespace(namespace) => namespace.get_span(),
      Structure::Function(function) => function.get_span(),
      Structure::TypeAlias(typealias) => typealias.get_span(),
      Structure::Interface(interface) => interface.get_span(),
      Structure::Struct(r#struct) => r#struct.get_span(),
      Structure::Class(class) => class.get_span(),
      Structure::Impl(r#impl) => r#impl.get_span(),
      Structure::Extern(r#extern) => r#extern.get_span(),
      Structure::Import(import) => import.get_span(),
      Structure::Exported(exported) => exported.get_span(),
      Structure::TemplateScope(templatescope) => templatescope.get_span(),
    }
  }
}

impl GetSpan for TemplateScope {
  fn get_span(&self) -> Span {
    self.span
  }
}

impl GetSpan for Exported {
  fn get_span(&self) -> Span {
    self.span
  }
}

impl MakeAst for Exported {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
        let Some(TokenEnum::Keyword(Keyword::Export)) = stream.next_variant() else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let Some(structure) = stream.make()? else {
      return ExpectedSnafu {
        what: "a structure",
        span: stream.span(),
      }.fail();
    };

    let structure = Box::new(structure);

    Ok(Some(Self {
      structure,
      span: stream.span_mark(),
    }))
  }
}

impl MakeAst for ConstrainedTemplateConstraint {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    let Some(ty) = stream.make()? else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let Some(TokenEnum::Keyword(Keyword::Extends)) = stream.next_variant() else {
      return Ok(None);
    };

    stream.skip_whitespace_and_comments();

    let Some(extends) = stream.make()? else {
      return ExpectedSnafu {
      what: "a type",
      span: stream.span(),
      }.fail();
    };

    Ok(Some(Self {
      ty,
      extends,
      span: stream.span_mark(),
    }))
  }
}

impl MakeAst for TemplateConstraint {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    #[allow(clippy::manual_map)]
    Ok({
      if let Some(extends) = stream.make()? {
        Some(Self::Extends(extends))
      } else if let Some(TokenEnum::Identifier(name)) = stream.peek_variant() {
        let name = name.to_owned();

        stream.seek();

        Some(Self::Unconstrained(UnconstrainedTemplateConstraint {
          name,
          span: stream.span_mark(),
        }))
      } else {
        None
      }
    })
  }
}

impl MakeAst for TemplatableStructure {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    #[allow(clippy::manual_map)]
    Ok({
      if let Some(exported) = stream.make()? {
        Some(Self::Exported(exported))
      } else if let Some(function) = stream.make()? {
        Some(Self::Function(function))
      } else if let Some(type_alias) = stream.make()? {
        Some(Self::TypeAlias(type_alias))
      } else if let Some(interface) = stream.make()? {
        Some(Self::Interface(interface))
      } else if let Some(r#struct) = stream.make()? {
        Some(Self::Struct(r#struct))
      } else if let Some(class) = stream.make()? {
        Some(Self::Class(class))
      } else if let Some(r#impl) = stream.make()? {
        Some(Self::Impl(r#impl))
      } else {
        None
      }
    })
  }
}

impl MakeAst for TemplateScope {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
        let Some(TokenEnum::Keyword(Keyword::Template)) = stream.next_variant() else {
      return Ok(None);
    };

        let Some(TokenEnum::Punctuation(Punctuation::Colon)) = stream.next_variant() else {
      return ExpectedSnafu {
        what: "a colon",
        span: stream.span(),
      }.fail();
    };

    stream.skip_whitespace_and_comments();

    let Some(constraint) = stream.make()? else {
      return ExpectedSnafu {
        what: "a template constraint",
        span: stream.span(),
      }.fail();
    };

    let mut constraints = vec![constraint];
    loop {
      stream.skip_whitespace_and_comments();

      match stream.next_variant() {
        Some(TokenEnum::Punctuation(Punctuation::Comma)) => {},
        Some(TokenEnum::Punctuation(Punctuation::Semicolon)) => break,
        _ => return ExpectedSnafu {
          what: "a semicolon or comma",
          span: stream.span(),
        }.fail()
      };

      let Some(constraint) = stream.make()? else {
        return ExpectedSnafu {
          what: "a template constraint",
          span: stream.span(),
        }.fail();
      };

      constraints.push(constraint);
    };

    stream.skip_whitespace_and_comments();

    let Some(structure) = stream.make()? else {
      return ExpectedSnafu {
        what: "a templatable structure",
        span: stream.span(),
      }.fail();
    };

    Ok(Some(Self {
      constraints,
      structure,
      span: stream.span_mark(),
    }))
  }
}

impl MakeAst for Structure {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    #[allow(clippy::manual_map)]
    Ok({
      if let Some(import) = stream.make()? {
        Some(Self::Import(import))
      } else if let Some(exported) = stream.make()? {
        Some(Self::Exported(exported))
      } else if let Some(ns) = stream.make()? {
        Some(Self::Namespace(ns))
      } else if let Some(func) = stream.make()? {
        Some(Self::Function(func))
      } else if let Some(alias) = stream.make()? {
        Some(Self::TypeAlias(alias))
      } else if let Some(alias) = stream.make()? {
        Some(Self::Interface(alias))
      } else if let Some(r#struct) = stream.make()? {
        Some(Self::Struct(r#struct))
      } else if let Some(class) = stream.make()? {
        Some(Self::Class(class))
      } else if let Some(r#impl) = stream.make()? {
        Some(Self::Impl(r#impl))
      } else if let Some(r#extern) = stream.make()? {
        Some(Self::Extern(r#extern))
      } else if let Some(scope) = stream.make()? {
        Some(Self::TemplateScope(scope))
      } else {
        None
      }
    })
  }
}
