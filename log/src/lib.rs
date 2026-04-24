use std::collections::HashMap;

use lazy_macros::colorize;

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
