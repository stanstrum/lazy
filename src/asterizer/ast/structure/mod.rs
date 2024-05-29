import_export!(namespace);
import_export!(type_alias);
import_export!(r#interface);
import_export!(r#struct);
import_export!(r#extern);

use typename::TypeName;

use crate::asterizer::ast::{
  MakeAst,
  TokenStream,
  AsterizerError,
  Function,
  Type
};

use crate::tokenizer::{
  TokenEnum,
  Keyword,
  Punctuation,
};

use crate::asterizer::error::ExpectedSnafu;

#[derive(Debug, TypeName)]
pub(crate) enum TemplatableStructure {
  Function(Function),
  TypeAlias(TypeAlias),
  Interface(Interface),
  Struct(Struct),
  Exported(Exported),
}

#[derive(Debug, TypeName)]
pub(crate) struct UnconstrainedTemplateConstraint {
  name: String
}

#[derive(Debug, TypeName)]
pub(crate) struct ConstrainedTemplateConstraint {
  ty: Type,
  extends: Type
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
  Extern(Extern),
  Exported(Exported),
  TemplateScope(TemplateScope),
}

#[derive(Debug, TypeName)]
pub(crate) struct TemplateScope {
  constraints: Vec<TemplateConstraint>,
  structure: TemplatableStructure
}

#[derive(Debug, TypeName)]
pub(crate) struct Exported {
  pub(crate) structure: Box<Structure>
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

    Ok(Some(Self { structure }))
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

    Ok(Some(Self { ty, extends }))
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

        Some(Self::Unconstrained(UnconstrainedTemplateConstraint { name }))
      } else {
        None
      }
    })
  }
}

impl TemplatableStructure {
  pub fn name(&self) -> String {
    match self {
      TemplatableStructure::Function(func) => func.decl.name.to_owned(),
      TemplatableStructure::TypeAlias(alias) => alias.name.to_owned(),
      TemplatableStructure::Interface(interface) => interface.name.to_owned(),
      TemplatableStructure::Struct(r#struct) => r#struct.name.to_owned(),
      TemplatableStructure::Exported(exported) => exported.structure.name(),
    }
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

    stream.skip_whitespace_and_comments();

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
        what: "a templatable structre",
        span: stream.span(),
      }.fail();
    };

    Ok(Some(Self { constraints, structure }))
  }
}

impl Structure {
  pub fn name(&self) -> String {
    match self {
      Self::Namespace(ns) => ns.name.to_owned(),
      Self::Function(func) => func.decl.name.to_owned(),
      Self::TypeAlias(alias) => alias.name.to_owned(),
      Self::Interface(r#interface) => r#interface.name.to_owned(),
      Self::Struct(r#struct) => r#struct.name.to_owned(),
      Self::Extern(r#extern) => r#extern.decl.name.to_owned(),
      Self::Exported(r#exported) => r#exported.structure.name(),
      Self::TemplateScope(scope) => scope.structure.name(),
    }
  }
}

impl MakeAst for Structure {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError> {
    #[allow(clippy::manual_map)]
    Ok({
      if let Some(exported) = stream.make()? {
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
