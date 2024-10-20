#[macro_export]
macro_rules! whitespace {
  () => { ' ' | '\t' | '\r' |  '\n' };
}

#[macro_export]
macro_rules! ident {
  () => { 'a'..='z' | 'A'..='Z' | '_' };
}

#[macro_export]
macro_rules! binary {
  () => { '0' | '1' };
}

#[macro_export]
macro_rules! octal {
  () => { '0'..='7' };
}

#[macro_export]
macro_rules! decimal {
  () => { '0'..='9' }
}

#[macro_export]
macro_rules! hexademical {
  () => { '0'..='9' | 'a'..='f' | 'A'..='F' };
}

#[macro_export]
macro_rules! operator {
  () => { '~' | '!' | '%' | '^' | '&' | '-' | '+' | '=' | '|' | '<' | '>' | '/' | '?' | ':' | ';' | ',' | '.' };
}
