use std::rc::Weak;

use super::*;

impl<'a> ParseScope<'a> for FunctionArgument {
  type In = ast::FunctionArgument<DefaultWorkflow>;
  type Scope = Function;

  fn parse_scope(
    translator: &mut Translator<DefaultWorkflow>,
    compiler: &Compiler<DefaultWorkflow>,
    input: Self::In,
    parent: &Option<WeakCell<Self::Scope>>,
  ) -> Result<RcCell<Self>> {
    let module = { parent.clone().unwrap().upgrade().unwrap().scope_parent() };

    let ty = translator.parse_scope(compiler, input.ty, &module)?;

    Ok(new_rc_cell(Self {
      name: input.identifier,
      ty,
      parent: parent.as_ref().cloned().unwrap().into(),
    }))
  }
}

impl BlockInstruction {
  fn parse(
    translator: &mut Translator<DefaultWorkflow>,
    compiler: &Compiler<DefaultWorkflow>,
    input: ast::BlockExpression<DefaultWorkflow>,
    parent: &WeakCell<FunctionBlock>,
  ) -> Result<Self> {
    // TODO: bad
    let child_parent = parent
      .upgrade()
      .unwrap()
      .scope_parent()
      .as_ref()
      .and_then(Weak::upgrade)
      .unwrap()
      .scope_parent();
    let parent = Some(parent.clone());

    let mut variables = vec![];
    let mut exprs = vec![];
    for child in input.children {
      match child {
        ast::BlockChild::Binding(binding) => match binding.kind {
          ast::BindingKind::OnlyType(ty) => {
            let ty = translator.parse_scope(compiler, ty, &child_parent)?;

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

    let variables: Vec<Rc<std::cell::RefCell<Variable>>> =
      variables.into_iter().map(new_rc_cell).collect::<Vec<_>>();

    let mut instructions = exprs
      .into_iter()
      .map(|expr| translator.parse_scope(compiler, expr, &parent))
      .collect::<Result<Vec<_>>>()?;

    if let Some(return_last) = input.return_last {
      let return_last = translator.parse_scope(compiler, return_last, &parent)?;
      instructions.push(new_rc_cell(Instruction::Return(Some(return_last))));
    };

    Ok(Self {
      parent: parent.unwrap().clone().into(),
      variables,
      instructions,
      span: input.span,
    })
  }
}

impl LiteralInstruction {
  fn parse(
    translator: &mut Translator<DefaultWorkflow>,
    compiler: &Compiler<DefaultWorkflow>,
    input: ast::Literal<DefaultWorkflow>,
    parent: &WeakCell<FunctionBlock>,
  ) -> Result<Self> {
    let kind = Rc::new(match input.kind {
      ast::LiteralKind::String(string_literal) => todo!(),
      ast::LiteralKind::Char(char_literal) => todo!(),
      ast::LiteralKind::Numeric(ast::NumericLiteral::Float(float)) => {
        LiteralInstructionKind::Float(float)
      },
      ast::LiteralKind::Numeric(ast::NumericLiteral::Generic(generic)) => {
        LiteralInstructionKind::Integer(generic)
      },
    });

    let ty = Type::UnresolvedInstrinsic(Rc::downgrade(&kind));

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

impl<'a> ParseScope<'a> for FunctionBlock {
  type In = ast::BlockExpression<DefaultWorkflow>;
  type Scope = Function;

  fn parse_scope(
    translator: &mut Translator<DefaultWorkflow>,
    compiler: &Compiler<DefaultWorkflow>,
    input: Self::In,
    parent: &Option<WeakCell<Self::Scope>>,
  ) -> Result<RcCell<Self>> {
    // instantiate self so our children have parent references
    let this = new_rc_cell(Self {
      parent: Some(parent.clone().unwrap()).into(),
      variables: vec![],
      children: vec![],
    });

    // parent reference for parsing children nodes
    let child_parent = Some(Rc::downgrade(&this));

    let mut exprs = vec![];
    // first, collect all of this scope's variables
    for child in input.children {
      // skip the statement if it's not a binding
      if let ast::BlockChild::Expression(expr) = child {
        exprs.push(expr);
      } else if let ast::BlockChild::Binding(binding) = child {
        let ty = match binding.kind {
          ast::BindingKind::OnlyType(ty) => {
            let scope_parent = parent.as_ref().unwrap().upgrade().unwrap().scope_parent();
            translator.parse_scope(compiler, ty, &scope_parent)?
          },
          ast::BindingKind::OnlyExpression(_) => {
            todo!()
          },
          ast::BindingKind::Both { .. } => {
            todo!()
          },
        };

        this.borrow_mut().variables.push(new_rc_cell(Variable {
          name: binding.identifier,
          ty,
        }));
      };
    }

    for expr in exprs {
      let instruction = translator.parse_scope(compiler, expr, &child_parent)?;
      this.borrow_mut().children.push(instruction);
    }

    if let Some(return_last) = input.return_last {
      let instruction = Instruction::parse_scope(translator, compiler, return_last, &child_parent)?;
      this
        .borrow_mut()
        .children
        .push(new_rc_cell(Instruction::Return(Some(instruction))));
    };

    Ok(this)
  }
}

impl<'a> ParseScope<'a> for Function {
  type In = ast::Function<DefaultWorkflow>;
  type Scope = Module;

  fn parse_scope(
    translator: &mut Translator<DefaultWorkflow>,
    compiler: &Compiler<DefaultWorkflow>,
    input: Self::In,
    parent: &Option<WeakCell<Self::Scope>>,
  ) -> Result<RcCell<Self>> {
    // SPONGE: this is a dummy block that gets destroyed when this scope ends --
    // this might lead to leaks or duplicates
    let body = new_rc_cell(FunctionBlock {
      parent: None.into(),
      variables: vec![],
      children: vec![],
    });

    let rc = new_rc_cell(Self {
      parent: parent.as_ref().cloned().into(),
      name: input.identifier,
      arguments: vec![],
      body,
      return_ty: new_rc_cell(Type::Intrinsic {
        kind: Intrinsic::Void,
        parent: parent.clone().unwrap().into(),
      }),
      generator_id: None,
    });

    let argument_parent = Some(Rc::downgrade(&rc));

    {
      rc.try_borrow().unwrap().body.borrow_mut().parent = argument_parent.clone().into();
    };

    let arguments = input
      .arguments
      .map(|x| x.arguments)
      .unwrap_or_default()
      .into_iter()
      .map(|argument| translator.parse_scope(compiler, argument, &argument_parent))
      .collect::<Result<_>>()?;

    if let Some(input) = input.return_ty {
      let return_ty = translator.parse_scope(compiler, input, parent)?;

      rc.borrow_mut().return_ty = return_ty;
    };

    {
      let mut function = rc.borrow_mut();

      function.parent = parent.clone().into();
      function.arguments = arguments;
    };

    // That dummy block from earlier gets dropped here
    rc.borrow_mut().body =
      FunctionBlock::parse_scope(translator, compiler, input.body, &argument_parent)?;

    Ok(rc)
  }
}
