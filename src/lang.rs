use std::collections::HashMap;

#[derive(Debug)]
pub enum Type {}

#[derive(Debug)]
pub struct Variable {
  name: String,
  ty: Type,
}

#[derive(Debug)]
pub struct Block {
  variables: Vec<Variable>,
}

#[derive(Debug)]
pub struct Function {
  arguments: Vec<Variable>,
  return_ty: Type,
  body: Block,
}

#[derive(Debug)]
pub struct Module {
  functions: HashMap<String, Function>,
  modules: HashMap<String, Module>,
}

#[derive(Debug)]
pub enum Structure {
  Module { name: String, module: Module },
  Function { name: String, function: Function },
}
