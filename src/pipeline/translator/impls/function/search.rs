use super::*;

use crate::compiler::error::ReadSpan;

impl SearchIn<FunctionBlock> for Variable {
  fn search_in(
    _scope: &FunctionBlock,
    _index: &<FunctionBlock as Scope>::Index,
  ) -> Result<ScopeSearch<Self, FunctionBlock>> {
    warn!("<Variable as SearchIn<FunctionBlock>>::search_in -- no one's home.");

    Ok(ScopeSearch::None)
  }
}

impl SearchIn<FunctionBlock> for LiteralInstruction {}

impl SearchIn<Function> for FunctionArgument {
  fn parent(&self) -> Option<WeakCell<Function>> {
    Some(self.parent.clone().unwrap())
  }

  fn search_in(
    scope: &Function,
    index: &<Function as Scope>::Index,
  ) -> Result<ScopeSearch<Self, Function>> {
    Ok(
      scope
        .arguments
        .iter()
        .find_map(|argument| {
          let name_matches = { argument.try_borrow().unwrap().name.name == index };

          name_matches.then(|| ScopeSearch::Found(Rc::downgrade(argument)))
        })
        .unwrap_or(ScopeSearch::None),
    )
  }
}

impl SearchIn<Module> for Function {
  fn parent(&self) -> Option<WeakCell<Module>> {
    self.parent.clone().unwrap()
  }

  fn search_in(
    scope: &Module,
    index: &<Module as Scope>::Index,
  ) -> Result<ScopeSearch<Self, Module>> {
    Ok(match ModuleChild::search_in(scope, index)? {
      ScopeSearch::Found(rc) => match &*rc.upgrade().unwrap().try_borrow().unwrap() {
        ModuleChild::Function(rc) => ScopeSearch::Found(Rc::downgrade(rc)),
        ModuleChild::Module(rc) => ScopeSearch::Next(Rc::downgrade(rc)),
        ModuleChild::Type(_) => todo!(),
      },
      ScopeSearch::Next(rc) => ScopeSearch::Next(rc),
      ScopeSearch::None => ScopeSearch::None,
    })
  }
}

impl SearchIn<Function> for FunctionBlock {
  fn parent(&self) -> Option<WeakCell<Function>> {
    self.parent.clone().unwrap()
  }
}

impl SearchIn<FunctionBlock> for BlockInstruction {
  fn parent(&self) -> Option<WeakCell<FunctionBlock>> {
    std::todo!()
  }

  fn search_in(scope: &FunctionBlock, index: &<FunctionBlock as Scope>::Index) -> Result<ScopeSearch<Self, FunctionBlock>> {
    std::todo!()
  }

  fn span(&self, compiler: &Compiler<DefaultWorkflow>) -> ReadSpan {
    std::todo!()
  }
}

impl SearchIn<FunctionBlock> for Instruction {
  fn parent(&self) -> Option<WeakCell<FunctionBlock>> {
    match self {
      Instruction::Literal(literal_instruction) => literal_instruction.parent(),
      Instruction::Block(block_instruction) => block_instruction.scope_parent(),
      Instruction::Return { parent, .. } => {
        Some(parent.as_ref().clone())
      },
      Instruction::ImplicitReturnLast { .. } => todo!(),
    }
  }

  fn search_in(scope: &FunctionBlock, index: &<FunctionBlock as Scope>::Index) -> Result<ScopeSearch<Self, FunctionBlock>> {
    todo!()
  }

  fn span(&self, compiler: &Compiler<DefaultWorkflow>) -> ReadSpan {
    todo!()
  }
}
