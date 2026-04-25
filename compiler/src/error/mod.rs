mod seek;
mod print;

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::fs::File;

use lazy_macros::{colorize, line_dbg};
use gluezy::{Lazy, ModuleReference};
use crate::lang::Span;

pub use print::print_message;

impl From<crate::aster::Error> for crate::lang::LazyError {
  fn from(value: crate::aster::Error) -> Self {
    Self::Aster(value)
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
