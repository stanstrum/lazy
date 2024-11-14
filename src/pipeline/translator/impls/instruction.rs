use std::rc::Weak;

use super::*;

impl BlockInstruction {
  fn parse(
    translator: &mut Translator<DefaultWorkflow>,
    compiler: &Compiler<DefaultWorkflow>,
    input: ast::BlockExpression<DefaultWorkflow>,
    parent: &WeakCell<FunctionBlock>,
  ) -> Result<RcCell<Self>> {
    // TODO: bad
    let child_parent = parent
      .upgrade().unwrap()
      .scope_parent().as_ref()
      .and_then(Weak::upgrade)
      .unwrap()
      .scope_parent();

    let mut variables = vec![];
    let mut exprs = vec![];
    for child in input.children {
      match child {
        ast::BlockChild::Binding(binding) => match binding.kind {
          ast::BindingKind::OnlyType(ty) => {
            let ty = translator.parse_scope::<Type<Module>, Module>(compiler, ty, &child_parent)?;

            variables.push(Variable {
              name: binding.identifier,
              ty,
            });
          },
          ast::BindingKind::OnlyExpression(expr) => {
            exprs.push(expr);
          },
          ast::BindingKind::Both { ty, expression } => {
            let ty = translator.parse_scope(compiler, ty, &child_parent)?;

            exprs.push(expression);
            variables.push(Variable {
              name: binding.identifier,
              ty,
            });
          },
        },
        ast::BlockChild::Expression(expr) => exprs.push(expr),
      };
    }

    let child_parent = Some(parent.clone());
    let variables: Vec<Rc<std::cell::RefCell<Variable>>> =
      variables.into_iter().map(new_rc_cell).collect::<Vec<_>>();

    let instructions = exprs
      .into_iter()
      .map(|expr| translator.parse_scope(compiler, expr, &child_parent))
      .collect::<Result<Vec<_>>>()?;

    let block = new_rc_cell(Self {
      parent: parent.clone().into(),
      variables,
      instructions,
      span: input.span,
    });

    if let Some(return_last) = input.return_last {
      let return_last = translator.parse_scope(compiler, return_last, &child_parent)?;
      let implicit_return = new_rc_cell(Instruction::ImplicitReturnLast {
        parent: parent.clone().into(),
        block: block.clone().into(),
        value: return_last,
      });

      block.borrow_mut().instructions.push(implicit_return);
    };

    Ok(block)
  }
}

impl LiteralInstruction {
  fn parse(
    _translator: &mut Translator<DefaultWorkflow>,
    _compiler: &Compiler<DefaultWorkflow>,
    input: ast::Literal<DefaultWorkflow>,
    parent: &WeakCell<FunctionBlock>,
  ) -> Result<Self> {
    let kind = Rc::new(match input.kind {
      ast::LiteralKind::String(_) => todo!(),
      ast::LiteralKind::Char(_) => todo!(),
      ast::LiteralKind::Numeric(ast::NumericLiteral::Float(float)) => {
        LiteralInstructionKind::Float(float)
      },
      ast::LiteralKind::Numeric(ast::NumericLiteral::Generic(generic)) => {
        LiteralInstructionKind::Integer(generic)
      },
    });

    let module = parent.upgrade().unwrap().scope_parent().unwrap().upgrade().unwrap().scope_parent().unwrap();
    let ty = new_rc_cell(Type::UnresolvedInstrinsic {
      weak: Rc::downgrade(&kind),
      parent: module.into(),
    });

    Ok(Self {
      kind,
      ty,
      span: input.span,
    })
  }
}

impl<'a> ParseScope<'a> for Instruction {
  type In = ast::Expression<DefaultWorkflow>;
  type Scope = FunctionBlock;

  fn parse_scope(
    translator: &mut Translator<DefaultWorkflow>,
    compiler: &Compiler<DefaultWorkflow>,
    input: Self::In,
    parent: &Option<WeakCell<Self::Scope>>,
  ) -> Result<RcCell<Self>> {
    let parent = parent.as_ref().unwrap();

    Ok(new_rc_cell(match input {
      ast::Expression::Block(block) => Self::Block(BlockInstruction::parse(
        translator, compiler, *block, parent,
      )?),
      ast::Expression::Literal(literal) => Self::Literal(LiteralInstruction::parse(
        translator, compiler, literal, parent,
      )?),
    }))
  }
}
