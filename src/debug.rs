use crate::asterizer::ast::GlobalNamespace;
use crate::tokenizer::Token;

pub(crate) fn tokens(toks: &Vec<Token>) {
  dbg!(toks);

  let source = toks.iter()
    .map(std::string::ToString::to_string)
    .reduce(|acc, e| acc + &e)
    .expect("failed to accumulate source code");

  println!("{source}");
}

pub(crate) fn ast(ast: &GlobalNamespace) {
  dbg!(&ast);
}
