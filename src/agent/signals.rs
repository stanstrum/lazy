use super::*;
use crate::translate::Translate;

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

    let err = loop {
      translator.skip_whitespace();

      match lang::Function::translate(&mut translator) {
        Ok(Some(())) => continue,
        Ok(None) => {},
        Err(err) => break Some(err.to_string()),
      };

      if translator.peek().is_none() {
        println!(
          "[thread #{id}] translation complete: {}",
          file.path.to_string_lossy()
        );

        break None;
      }

      todo!("dunno what to do")
    };

    if let Some(err) = err {
      tx.send(CompilerSignal::AgentError {
        id,
        err: err.to_string(),
      })
      .unwrap();
      panic!("error");
    };
  }
}
