mod seek;
mod print;

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::fs::File;

use crate::{colorize, line_dbg};
use crate::lang::Lazy;
use crate::lang::reference::ModuleReference;
use crate::tokenize::token::Span;

pub use print::print_message;

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
pub struct PrintableMessage {
  pub level: Level,
  pub force: bool,
  pub description: String,
  pub contents: MessageContents,
}

#[derive(Debug)]
pub struct WithinSource {
  pub range: Span,
  pub sections: Vec<MessageSection>,
}

#[derive(Debug)]
pub enum MessageContents {
  WithinSource(Vec<WithinSource>),
  File(ModuleReference),
  None,
}

#[derive(Debug)]
pub struct MessageSection {
  pub text: String,
  pub span: Span,
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

impl WithinSource {
  pub fn new(sources: Vec<MessageSection>) -> Vec<Self> {
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

impl From<crate::tokenize::Error> for PrintableMessage {
  fn from(value: crate::tokenize::Error) -> Self {
    match value {
      crate::tokenize::Error::IO { name, module } => Self {
        level: Level::Error,
        force: true,
        description: format!(line_dbg!("IO error for {}"), name),
        contents: MessageContents::File(module),
      },
      crate::tokenize::Error::InvalidNumeric { span } => Self {
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

impl From<crate::aster::Error> for PrintableMessage {
  fn from(value: crate::aster::Error) -> Self {
    match value {
      crate::aster::Error::Token(error) => error.into(),
      crate::aster::Error::Expected { what, at } => Self {
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
      crate::aster::Error::Invalid { what, at } => Self {
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
      crate::aster::Error::Lazy(lazy) => (*lazy).into(),
    }
  }
}

impl From<Box<crate::resolve::Error>> for PrintableMessage {
  fn from(value: Box<crate::resolve::Error>) -> Self {
    #[cfg(debug_assertions)]
    println!("{}", value.call_stack);

    match value.base {
      crate::resolve::ErrorBase::UnknownTypeName { module_name, span } => Self {
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
      crate::resolve::ErrorBase::MissingEntryPoint { module_name, file } => Self {
        level: Level::Error,
        force: true,
        description: format!(line_dbg!("{:?} is missing an entry point!"), module_name),
        contents: MessageContents::File(file),
      },
      crate::resolve::ErrorBase::TypeMismatch {
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
      crate::resolve::ErrorBase::UnresolvedInVerify { what, span } => Self {
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
      crate::resolve::ErrorBase::BadQualify { span } => Self {
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
      crate::resolve::ErrorBase::NotImplemented { what, span } => Self {
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
      crate::resolve::ErrorBase::Lazy(lazy_error) => (*lazy_error).into(),
    }
  }
}

impl From<crate::generate::Error> for PrintableMessage {
  fn from(value: crate::generate::Error) -> Self {
    match value {
      crate::generate::Error::StillUnresolved { what, note, span } => Self {
        level: Level::Error,
        force: true,
        description: format!("unresolved in generation: {what}"),
        contents: MessageContents::WithinSource(WithinSource::new(
          vec![MessageSection {
            text: note,
            span,
          }],
        )),
      },
      crate::generate::Error::LLVMError(description) => Self {
        level: Level::Error,
        force: true,
        description,
        contents: MessageContents::None,
      },
    }
  }
}

impl From<crate::lang::LazyError> for PrintableMessage {
  fn from(value: crate::lang::LazyError) -> Self {
    match value {
      crate::lang::LazyError::NotExist(path_buf) => Self {
        level: Level::Error,
        force: true,
        description: format!(line_dbg!("not a file {:?}"), &path_buf),
        contents: MessageContents::None,
      },
      crate::lang::LazyError::Aster(aster) => aster.into(),
    }
  }
}
