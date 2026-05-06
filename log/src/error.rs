use super::*;

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

impl<C: Compiler> From<lang::error::AsterError<C>> for PrintableMessage<C> {
  fn from(value: lang::error::AsterError<C>) -> Self {
    match value {
      lang::error::AsterError::Token(error) => error.into(),
      lang::error::AsterError::Expected { what, at } => Self {
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
      lang::error::AsterError::Invalid { what, at } => Self {
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
      lang::error::AsterError::Lazy(lazy) => (*lazy).into(),
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

impl<C: Compiler> From<lang::error::LazyError<C>> for PrintableMessage<C> {
  fn from(value: lang::error::LazyError<C>) -> Self {
    match value {
      lang::error::LazyError::NotExist(path_buf) => Self {
        level: Level::Error,
        force: true,
        description: format!(line_dbg!("not a file {:?}"), &path_buf),
        contents: MessageContents::None,
      },
      lang::error::LazyError::Aster(aster) => aster.into(),
    }
  }
}
