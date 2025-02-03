use super::*;
use crate::{
  token::{Punctuation, TokenKind},
  translate::Translate,
};

#[derive(Debug)]
pub(crate) struct TranslateSignal(pub LazyFile, pub usize);

impl AgentDispatch for TranslateSignal {
  fn run(self: Box<Self>, id: usize, tx: &Sender<CompilerSignal>) {
    let TranslateSignal(file, fid) = *self;
    let chars = file.source_chars().unwrap();
    let tokens = token::Tokenizer::new(fid, chars)
      .enumerate()
      .map(|(index, token)| {
        println!("[thread #{id}] token[{index}] = {token:?}");
        token
      });

    let mut translator = translate::Translator::new(fid, tokens, tx);

    let mut err = 'err: {
      while {
        translator.skip_whitespace();
        translator.peek().is_some()
      } {
        match lang::Function::translate(&mut translator) {
          Ok(Some(())) => continue,
          Ok(None) => {},
          Err(err) => break 'err Some(err.to_string()),
        };

        let Some((_, peek_span)) = translator.peek() else {
          println!(
            "[translate] file {} broke from EOF",
            file.path.to_string_lossy()
          );
          break 'err None;
        };

        let peek_mark = peek_span.end;

        translator.skip_whitespace();

        let Some((next_kind, next_peek_span)) = translator.peek() else {
          println!(
            "[translate] file {} broke from EOF after whitespace",
            file.path.to_string_lossy()
          );
          break 'err None;
        };

        if let TokenKind::Punctuation(Punctuation::Semicolon) = &next_kind {
          translator.seek();
          continue;
        };

        assert!(
          next_peek_span.start.line > peek_mark.line,
          "expected a newline; probably a parse error"
        );
        assert!(
          next_peek_span.end.indentation_level == peek_mark.indentation_level,
          "we missed something big"
        );

        // otherwise continue
      }

      None
    };

    if translator.peek().is_some() {
      println!(
        "[thread #{id}] file {}: extra tokens",
        file.path.to_string_lossy()
      );
      let mut start = None;
      let mut end = None;
      let mut counter = 0;
      while let Some((kind, span)) = translator.next() {
        start.get_or_insert((span.start.start_of_line_byte, span.start.column));
        end.get_or_insert((span.end.start_of_line_byte, span.end.column));
        println!("[thread #{id}] id #{fid} extra[{counter}]: {kind:?}");
        println!("[thread #{id}] id #{fid} extra[{counter}]: {span:?}");

        counter += 1;
      }

      err.get_or_insert_with(|| format!("{counter} extra tokens !!!"));
    };

    if let Some(err) = err {
      tx.send(CompilerSignal::AgentError { id, err }).unwrap();
    };
  }
}
