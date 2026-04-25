use std::collections::HashMap;

use lazy_macros::{colorize, line_dbg};

use lang::{Compiler, span::Span};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(u8)]
#[allow(unused)]
pub enum Level {
  Debug,
  Stub,
  Info,
  Warn,
  Error,
}

#[derive(Debug)]
pub struct PrintableMessage<C: Compiler> {
  pub level: Level,
  pub force: bool,
  pub description: String,
  pub contents: MessageContents<C>,
}

#[derive(Debug)]
pub struct WithinSource<C: Compiler> {
  pub range: Span<C>,
  pub sections: Vec<MessageSection<C>>,
}

#[derive(Debug)]
pub enum MessageContents<C: Compiler> {
  WithinSource(Vec<WithinSource<C>>),
  File(C::ModuleReference),
  None,
}

#[derive(Debug)]
pub struct MessageSection<C: Compiler> {
  pub text: String,
  pub span: Span<C>,
}

impl std::fmt::Display for Level {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(match self {
      Level::Debug => concat!(              colorize!(36), "debug", colorize!(0)),
      Level::Stub =>  concat!(colorize!(7), colorize!(3),  "stub" , colorize!(0)),
      Level::Info =>  concat!(colorize!(7), colorize!(92), "info" , colorize!(0)),
      Level::Warn =>  concat!(colorize!(7), colorize!(93), "warn" , colorize!(0)),
      Level::Error => concat!(colorize!(7), colorize!(91), "error", colorize!(0)),
    })
  }
}

impl<C: Compiler> WithinSource<C> {
  pub fn new(sources: Vec<MessageSection<C>>) -> Vec<Self> {
    let mut map = HashMap::new();

    for section in sources {
      let list = map.entry(section.span.module).or_insert_with(Vec::new);
      list.push(section);
    };

    let mut results = Vec::with_capacity(map.len());
    for mut sources in map.into_values() {
      sources.sort_by_key(|section| section.span.start.position);

      assert!(!sources.is_empty());
      let start = sources.first().unwrap().span;
      let end = sources.last().unwrap().span;

      let range = Span::from_pair(start, end);

      results.push(WithinSource {
        range,
        sections: sources,
      });
    };

    results
  }
}

pub fn print_message<C: Compiler>(store: &C::Store<'_>, message: PrintableMessage<C>) {
  todo!()
}

impl<C: Compiler> From<lang::error::TokenError<C>> for PrintableMessage<C> {
  fn from(value: lang::error::TokenError<C>) -> Self {
    match value {
      lang::error::TokenError::IO { name, module } => Self {
        level: Level::Error,
        force: true,
        description: format!(line_dbg!("IO error for {}"), name),
        contents: MessageContents::File(module),
      },
      lang::error::TokenError::InvalidNumeric { span } => Self {
        level: Level::Error,
        force: true,
        description: line_dbg!("Invalid numeric").into(),
        contents: MessageContents::WithinSource(vec![WithinSource {
          range: span,
          sections: vec![MessageSection {
            text: "here".into(),
            span,
          }],
        }]),
      },
    }
  }
}

impl<C: Compiler> From<::lang::error::AsterError<C>> for PrintableMessage<C> {
  fn from(value: ::lang::error::AsterError<C>) -> Self {
    match value {
      ::lang::error::AsterError::Token(error) => error.into(),
      ::lang::error::AsterError::Expected { what, at } => Self {
        level: Level::Error,
        force: true,
        description: format!(line_dbg!("expected {}"), what),
        contents: MessageContents::WithinSource(vec![WithinSource {
          range: at,
          sections: vec![MessageSection {
            text: "here".into(),
            span: at,
          }],
        }]),
      },
      ::lang::error::AsterError::Invalid { what, at } => Self {
        level: Level::Error,
        force: true,
        description: format!(line_dbg!("invalid {}"), what),
        contents: MessageContents::WithinSource(vec![WithinSource {
          range: at,
          sections: vec![MessageSection {
            text: "here".into(),
            span: at,
          }],
        }]),
      },
      ::lang::error::AsterError::Lazy(lazy) => (*lazy).into(),
    }
  }
}

impl<C: Compiler> From<Box<lang::error::ResolveError<C>>> for PrintableMessage<C> {
  fn from(value: Box<lang::error::ResolveError<C>>) -> Self {
    #[cfg(debug_assertions)]
    println!("{}", value.call_stack);

    match value.base {
      lang::error::ResolveErrorBase::UnknownTypeName { module_name, span } => Self {
        level: Level::Error,
        force: true,
        description: format!(line_dbg!("unknown type name in {}"), module_name),
        contents: MessageContents::WithinSource(vec![WithinSource {
          range: span,
          sections: vec![MessageSection {
            text: "here".into(),
            span,
          }],
        }]),
      },
      lang::error::ResolveErrorBase::MissingEntryPoint { module_name, file } => Self {
        level: Level::Error,
        force: true,
        description: format!(line_dbg!("{:?} is missing an entry point!"), module_name),
        contents: MessageContents::File(file),
      },
      lang::error::ResolveErrorBase::TypeMismatch {
        whence,
        a_print, a_span,
        b_print, b_span,
      } => {
        let a_section = MessageSection {
          text: "here".into(),
          span: a_span,
        };

        let b_section = MessageSection {
          text: "here".into(),
          span: b_span,
        };

        let mut sections = vec![a_section];

        if a_span != b_span {
          sections.push(b_section);
        };

        let within_sources = WithinSource::new(sections);

        Self {
          level: Level::Error,
          force: true,
          description: format!(line_dbg!("{}{} is not coercible with {}"), whence, a_print, b_print),
          contents: MessageContents::WithinSource(within_sources),
        }
      },
      lang::error::ResolveErrorBase::UnresolvedInVerify { what, span } => Self {
        level: Level::Error,
        force: true,
        description: format!(line_dbg!("verify error: {} is not resolved"), what),
        contents: MessageContents::WithinSource(WithinSource::new(
          vec![MessageSection {
            text: "here".into(),
            span,
          }],
        )),
      },
      lang::error::ResolveErrorBase::BadQualify { span } => Self {
        level: Level::Error,
        force: true,
        description: line_dbg!("invalid part in qualifier").into(),
        contents: MessageContents::WithinSource(WithinSource::new(
          vec![MessageSection {
            text: "here".into(),
            span,
          }],
        )),
      },
      lang::error::ResolveErrorBase::NotImplemented { what, span } => Self {
        level: Level::Error,
        force: true,
        description: format!("not implemented: {what}"),
        contents: MessageContents::WithinSource(WithinSource::new(
          vec![MessageSection {
            text: "here".into(),
            span,
          }],
        )),
      },
      lang::error::ResolveErrorBase::Lazy(lazy_error) => (*lazy_error).into(),
    }
  }
}

impl<C: Compiler> From<::lang::error::LazyError<C>> for PrintableMessage<C> {
  fn from(value: ::lang::error::LazyError<C>) -> Self {
    match value {
      ::lang::error::LazyError::NotExist(path_buf) => Self {
        level: Level::Error,
        force: true,
        description: format!(line_dbg!("not a file {:?}"), &path_buf),
        contents: MessageContents::None,
      },
      ::lang::error::LazyError::Aster(aster) => aster.into(),
    }
  }
}
