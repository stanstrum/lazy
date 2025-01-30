use super::*;

#[derive(Debug)]
pub(crate) struct Translate(pub LazyFile);

impl AgentDispatch for Translate {
  fn run(self: Box<Self>, id: usize, tx: &Sender<CompilerSignal>) {
    let Translate(file) = *self;
    let chars = file.source_chars().unwrap();
    let tokens = token::Tokenizer::new(id, chars);

    for (index, token) in tokens.enumerate() {
      println!("[thread #{id}] token[{index}] = {token:?}");
    }

    todo!();
  }
}
