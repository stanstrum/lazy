use std::collections::HashMap;

use std::rc::Rc;
use std::cell::RefCell;

use crate::tokenizer::{GetSpan, Span};
use crate::asterizer::ast;

use crate::typechecker::lang::GenericConstraints;
use crate::typechecker::{
  check::Extends,
  Domain,
  DomainMember,
  DomainMemberKind,
  error::*,
  NamedDomainMember,
  Preprocessor,
  TypeOf,
};

use crate::typechecker::lang::{
  Block,
  ControlFlow,
  ExternFunction,
  Function,
  Instruction,
  pretty_print::PrettyPrint,
  Type,
  TypeCell,
  Value,
  Variable,
  VariableKind,
  VariableScope,
  VariableReference,
  intrinsics::Intrinsic,
};

use super::lang::{GenericConstraint, Struct};

pub(super) trait PreprocessExpression {
  type Out;

  fn preprocess(&self, preprocessor: &mut Preprocessor, return_ty: &TypeCell) -> Result<Self::Out, TypeCheckerError>;
}

pub(super) trait Preprocess {
  type Out;

  fn preprocess(&self, preprocessor: &mut Preprocessor) -> Result<Self::Out, TypeCheckerError>;
}

impl Preprocess for ast::Atom {
  type Out = Instruction;

  fn preprocess(&self, preprocessor: &mut Preprocessor) -> Result<Self::Out, TypeCheckerError> {
    Ok({
      match self {
        ast::Atom::Literal(literal) => {
          Instruction::Value(
            Value::Literal {
              literal: literal.to_owned(),
              ty: literal.type_of_or_unknown().into(),
            }
          )
        },
        ast::Atom::StructInitializer(_) => todo!("preprocess structinitializer"),
        ast::Atom::Variable { name, span } => {
          Instruction::Value(Value::Variable(
            preprocessor.find_variable_by_name(&name, *span)?
          ))
        },
      }
    })
  }
}

impl PreprocessExpression for ast::Expression {
  type Out = Instruction;

  fn preprocess(&self, preprocessor: &mut Preprocessor, return_ty: &TypeCell) -> Result<Self::Out, TypeCheckerError> {
    Ok({
      match self {
        ast::Expression::Atom(atom) => atom.preprocess(preprocessor)?,
        ast::Expression::Block(block) => Instruction::Block(block.preprocess(preprocessor, return_ty)?),
        ast::Expression::SubExpression(_) => todo!("preprocess subexpression"),
        ast::Expression::Unary(_) => todo!("preprocess unary"),
        ast::Expression::Binary(_) => todo!("preprocess binary"),
      }
    })
  }
}

impl PreprocessExpression for ast::If {
  type Out = Instruction;

  fn preprocess(&self, _preprocessor: &mut Preprocessor, _return_ty: &TypeCell) -> Result<Self::Out, TypeCheckerError> {
    todo!()
  }
}

impl PreprocessExpression for ast::While {
  type Out = Instruction;

  fn preprocess(&self, preprocessor: &mut Preprocessor, return_ty: &TypeCell) -> Result<Self::Out, TypeCheckerError> {
    Ok(Instruction::ControlFlow(ControlFlow::While {
      condition: Value::Instruction(Box::new(
        self.clause.preprocess(preprocessor, return_ty)?
      )),
      body: self.body.preprocess(preprocessor, return_ty)?,
      span: self.span,
    }))
  }
}

impl PreprocessExpression for ast::DoWhile {
  type Out = Instruction;

  fn preprocess(&self, preprocessor: &mut Preprocessor, return_ty: &TypeCell) -> Result<Self::Out, TypeCheckerError> {
    Ok(Instruction::ControlFlow(ControlFlow::DoWhile {
      condition: Value::Instruction(Box::new(
        self.clause.preprocess(preprocessor, return_ty)?
      )),
      body: self.body.preprocess(preprocessor, return_ty)?,
      span: self.span,
    }))
  }
}

impl PreprocessExpression for ast::Until {
  type Out = Instruction;

  fn preprocess(&self, preprocessor: &mut Preprocessor, return_ty: &TypeCell) -> Result<Self::Out, TypeCheckerError> {
    Ok(Instruction::ControlFlow(ControlFlow::Until {
      condition: Value::Instruction(Box::new(
        self.clause.preprocess(preprocessor, return_ty)?
      )),
      body: self.body.preprocess(preprocessor, return_ty)?,
      span: self.span,
    }))
  }
}

impl PreprocessExpression for ast::DoUntil {
  type Out = Instruction;

  fn preprocess(&self, preprocessor: &mut Preprocessor, return_ty: &TypeCell) -> Result<Self::Out, TypeCheckerError> {
    Ok(Instruction::ControlFlow(ControlFlow::Until {
      condition: Value::Instruction(Box::new(
        self.clause.preprocess(preprocessor, return_ty)?
      )),
      body: self.body.preprocess(preprocessor, return_ty)?,
      span: self.span,
    }))
  }
}

impl PreprocessExpression for ast::For {
  type Out = Instruction;

  fn preprocess(&self, _preprocessor: &mut Preprocessor, _return_ty: &TypeCell) -> Result<Self::Out, TypeCheckerError> {
    todo!()
  }
}

impl PreprocessExpression for ast::Loop {
  type Out = Instruction;

  fn preprocess(&self, _preprocessor: &mut Preprocessor, _return_ty: &TypeCell) -> Result<Self::Out, TypeCheckerError> {
    todo!()
  }
}

impl PreprocessExpression for ast::ControlFlow {
  type Out = Instruction;

  fn preprocess(&self, preprocessor: &mut Preprocessor, return_ty: &TypeCell) -> Result<Self::Out, TypeCheckerError> {
    match self {
      ast::ControlFlow::If(r#if) => r#if.preprocess(preprocessor, return_ty),
      ast::ControlFlow::While(r#while) => r#while.preprocess(preprocessor, return_ty),
      ast::ControlFlow::DoWhile(dowhile) => dowhile.preprocess(preprocessor, return_ty),
      ast::ControlFlow::Until(until) => until.preprocess(preprocessor, return_ty),
      ast::ControlFlow::DoUntil(dountil) => dountil.preprocess(preprocessor, return_ty),
      ast::ControlFlow::For(r#for) => r#for.preprocess(preprocessor, return_ty),
      ast::ControlFlow::Loop(r#loop) => r#loop.preprocess(preprocessor, return_ty),
    }
  }
}

impl PreprocessExpression for ast::BlockChild {
  type Out = Option<Instruction>;

  fn preprocess(&self, preprocessor: &mut Preprocessor, return_ty: &TypeCell) -> Result<Self::Out, TypeCheckerError> {
    Ok({
      match self {
        Self::Binding(binding) => {
          if let Some(expr) = &binding.expr {
            let reference = preprocessor.find_variable_by_name(&binding.name, binding.get_span())?;

            let value = Value::Instruction(Box::new(
              expr.preprocess(preprocessor, return_ty)?
            ));

            Some(Instruction::Assign {
              dest: Value::Variable(reference),
              value,
              span: expr.get_span().to_owned(),
            })
          } else {
            None
          }
        },
        Self::Expression(expr) => {
          Some(expr.preprocess(preprocessor, return_ty)?)
        },
        Self::ControlFlow(ctrl_flow) => Some(ctrl_flow.preprocess(preprocessor, return_ty)?),
        Self::Return(ast::Return { expr, span, .. }) => {
          let value = if let Some(expr) = &expr {
            Some(Value::Instruction(Box::new(
              expr.preprocess(preprocessor, return_ty)?
            )))
          } else {
            None
          };

          Some(Instruction::Return {
            value,
            span: span.to_owned(),
            to: return_ty.to_owned(),
          })
        },
      }
    })
  }
}

impl PreprocessExpression for ast::Block {
  type Out = Block;

  fn preprocess(&self, preprocessor: &mut Preprocessor, return_ty: &TypeCell) -> Result<Self::Out, TypeCheckerError> {
    let mut variables = vec![];
    let mut body = vec![];

    let mut variable_map = HashMap::new();

    if self.returns_last {
      match self.children.last() {
        Some(ast::BlockChild::Binding(last_binding)) => return InvalidSnafu {
          message: "a block returning the last statement may not end with an assignment",
          span: last_binding.get_span(),
        }.fail(),
        None => return InvalidSnafu {
          message: "a block returning the last statement must have at least one instruction",
          span: self.span,
        }.fail(),
        _ => {},
      };
    };

    for child in self.children.iter() {
      if let ast::BlockChild::Binding(binding) = child {
        let variable_id = variables.len();

        variable_map.insert(&binding.name, variable_id);
        variables.push(Variable {
          name: binding.name.to_owned(),
          kind: VariableKind::LocalVariable,
          ty: {
            if let Some(binding_type) = &binding.ty {
              binding_type.preprocess(preprocessor)?
            } else {
              Type::Unknown {
                span: binding.span,
              }
            }.into()
          },
          span: binding.span,
        });
      };
    };

    let scope = Rc::new(RefCell::new(VariableScope::from_vec(variables)));
    let inner = &scope.borrow().inner;

    let variable_scope =
      variable_map.into_iter().map(
        |(name, id)| {
          let span = {
            inner.get(id).unwrap().get_span().to_owned()
          };

          (
            name.to_owned(),
            VariableReference {
              scope: scope.to_owned(),
              id,
              span,
            }
          )
        });

    preprocessor.scope_stack.push(variable_scope.collect());

    for child in self.children.iter() {
      if let Some(instruction) = child.preprocess(preprocessor, return_ty)? {
        body.push(instruction)
      };
    };

    preprocessor.scope_stack.pop();

    Ok(Self::Out {
      // TODO: refactor here
      variables: scope.to_owned(),
      body,
      span: self.span.to_owned(),
      returns_last: self.returns_last,
    })
  }
}

impl Preprocess for ast::Function {
  type Out = Function;

  fn preprocess(&self, preprocessor: &mut Preprocessor) -> Result<Self::Out, TypeCheckerError> {
    let mut arguments = vec![];

    if let Some(decl_args) = &self.decl.args {
      for arg in decl_args.args.iter() {
        arguments.push(Variable {
          name: arg.name.to_owned(),
          kind: VariableKind::Argument,
          ty: arg.ty.preprocess(preprocessor)?.into(),
          span: arg.span.to_owned(),
        });
      };
    };

    let arguments = Rc::new(RefCell::new(VariableScope::from_vec(arguments)));

    preprocessor.scope_stack.push(
      arguments.borrow().inner.iter().enumerate().map(|(id, var)| (
        var.name.to_owned(),
        VariableReference {
          scope: arguments.to_owned(),
          id,
          span: var.get_span(),
        }
      )).collect()
    );

    let return_ty = if let Some(ty) = &self.decl.return_type {
      ty.preprocess(preprocessor)?
    } else {
      Type::Intrinsic {
        kind: Intrinsic::Void,
        span: self.decl.get_span().to_owned(),
      }
    }.into();

    let mut body = self.body.preprocess(preprocessor, &return_ty)?;

    if self.body.returns_last {
      let Some(last) = body.body.pop() else {
        return IncompatibleTypesSnafu {
          message: "body returns last, but has no instructions",
          lhs: return_ty.pretty_print(),
          rhs: "(void)",
          span: body.span,
        }.fail();
      };

      body.body.push(Instruction::Return {
        span: last.get_span(),
        value: Some(Value::Instruction(Box::new(last))),
        to: return_ty.to_owned(),
      });
    };

    if
      return_ty.borrow().extends(&Type::Intrinsic { kind: Intrinsic::Void, span: self.span }) &&
      !matches!(body.body.last(), Some(Instruction::Return { .. }))
    {
      body.body.push(Instruction::Return {
        value: None,
        to: return_ty.to_owned(),
        span: return_ty.get_span(),
      });
    };

    Ok(Function {
      name: self.decl.name.to_owned(),
      arguments,
      return_ty,
      body,
      span: self.span.to_owned(),
    })
  }
}

impl Preprocess for ast::Extern {
  type Out = ExternFunction;

  fn preprocess(&self, preprocessor: &mut Preprocessor) -> Result<Self::Out, TypeCheckerError> {
    let arguments = if let Some(args) = &self.decl.args {
      args.args.iter()
        .map(|arg| Ok(Variable {
          name: arg.name.to_owned(),
          kind: VariableKind::Argument,
          ty: arg.ty.preprocess(preprocessor)?.into(),
          span: arg.get_span(),
        })).collect::<Result<Vec<_>, _>>()?
    } else {
      vec![]
    };

    let return_ty = if let Some(return_ty) = &self.decl.return_type {
      return_ty.preprocess(preprocessor)?
    } else {
      Type::Intrinsic {
        kind: Intrinsic::Void,
        span: self.decl.get_span(),
      }
    }.into();

    Ok(ExternFunction {
      name: self.decl.name.to_owned(),
      arguments: Rc::new(RefCell::new(VariableScope::from_vec(arguments))),
      return_ty,
      span: self.span,
      variadic: self.c_variadic,
    })
  }
}

impl Preprocess for ast::Struct {
  type Out = NamedDomainMember;

  fn preprocess(&self, preprocessor: &mut Preprocessor) -> Result<Self::Out, TypeCheckerError> {
    Ok(NamedDomainMember {
      name: self.name.to_owned(),
      member: DomainMember {
        kind: DomainMemberKind::Struct(Struct {
          members: Rc::new(RefCell::new(
            self.members.iter()
              .map(|memb| {
                Ok(memb.ty.preprocess(preprocessor)?.into())
              })
              .collect::<Result<Vec<_>, _>>()?
          )),
          span: self.get_span(),
        }),
        template_scope: None,
      },
    })
  }
}

impl Preprocess for ast::TemplateConstraint {
  type Out = (String, Vec<GenericConstraint>);

  fn preprocess(&self, preprocessor: &mut Preprocessor) -> Result<Self::Out, TypeCheckerError> {
    match self {
      ast::TemplateConstraint::Unconstrained(unconstrained) => Ok((
        unconstrained.name.to_owned(),
        vec![],
      )),
      ast::TemplateConstraint::Extends(extends) => {
        let lhs = extends.ty.preprocess(preprocessor)?;
        let rhs = extends.ty.preprocess(preprocessor)?;

        let Type::Unresolved { reference, implied: false, .. } = &lhs else {
          panic!("invalid template type");
        };

        let Some(name) = reference.inner.first().map(Clone::clone) else {
          panic!("invalid template type");
        };

        Ok((
          name,
          vec![
            GenericConstraint::Extends {
              lhs: lhs.into(),
              rhs: rhs.into(),
              span: self.get_span(),
            }
          ]
        ))
      },
    }
  }
}

impl Preprocess for Vec<ast::TemplateConstraint> {
  type Out = Rc<RefCell<Vec<(String, TypeCell)>>>;

  // TODO: refactor this entirely
  fn preprocess(&self, preprocessor: &mut Preprocessor) -> Result<Self::Out, TypeCheckerError> {
    self.iter()
      .map(|constraint| {
        let (name, constraints) = constraint.preprocess(preprocessor)?;

        Ok((
          name,
          Type::Generic {
            constraints: GenericConstraints(constraints),
            span: constraint.get_span(),
          }.into(),
        ))
      })
      .collect::<Result<Vec<_>, _>>()
      .map(|result| Rc::new(RefCell::new(result)))
  }
}

impl Preprocess for ast::TemplateScope {
  type Out = Option<NamedDomainMember>;

  fn preprocess(&self, preprocessor: &mut Preprocessor) -> Result<Self::Out, TypeCheckerError> {
    let mut named_constraints: Vec<(String, Vec<GenericConstraint>, Span)> = vec![];

    for constraint in self.constraints.iter() {
      let (name, constraints) = constraint.preprocess(preprocessor)?;

      // TODO: there must be a better way to do this, preserving order
      if let Some(index) = 'index: {
        for (index, (existing_name, ..)) in named_constraints.iter().enumerate() {
          if name == **existing_name {
            break 'index Some(index);
          };
        };

        None
      } {
        let (_, existing_constraints, ..) = &mut named_constraints[index];

        existing_constraints.extend(constraints);
      } else {
        named_constraints.push((name, constraints, constraint.get_span()));
      };
    };

    preprocessor.template_scopes.push(
      named_constraints.into_iter().map(|(name, constraints, span)| (
        name,
        Type::Generic {
          constraints: GenericConstraints(constraints),
          span,
        }
      )).collect()
    );

    let mut named_domain_member = match &self.structure {
      ast::TemplatableStructure::Function(_) => todo!(),
      ast::TemplatableStructure::TypeAlias(_) => todo!(),
      ast::TemplatableStructure::Interface(_) => todo!(),
      ast::TemplatableStructure::Struct(r#struct) => {
        r#struct.preprocess(preprocessor)?
      },
      ast::TemplatableStructure::Class(_) => todo!(),
      ast::TemplatableStructure::Exported(_) => todo!(),
      ast::TemplatableStructure::Impl(_) => todo!(),
    };

    named_domain_member.member.template_scope = Some(self.constraints.preprocess(preprocessor)?);

    Ok(Some(named_domain_member))
  }
}

impl Preprocess for ast::Structure {
  type Out = Option<NamedDomainMember>;

  fn preprocess(&self, preprocessor: &mut Preprocessor) -> Result<Self::Out, TypeCheckerError> {
    Ok({
      match self {
        ast::Structure::Namespace(_) => todo!("preprocess namespace"),
        ast::Structure::Function(func) => {
          Some(NamedDomainMember {
            name: func.decl.name.to_owned(),
            member: DomainMember {
              kind: DomainMemberKind::Function(
                func.preprocess(preprocessor)?
              ),
              template_scope: None,
            }
          })
        },
        ast::Structure::TypeAlias(alias) => {
          Some(NamedDomainMember {
            name: alias.name.to_owned(),
            member: DomainMember {
              kind: DomainMemberKind::Type(alias.ty.preprocess(preprocessor)?.into()),
              template_scope: None,
            },
          })
        },
        ast::Structure::Interface(_) => todo!("preprocess interface"),
        ast::Structure::Struct(_) => todo!("preprocess struct"),
        ast::Structure::Class(_) => todo!("preprocess class"),
        ast::Structure::Extern(r#extern) => {
          Some(NamedDomainMember {
            name: r#extern.decl.name.to_owned(),
            member: DomainMember {
              kind: DomainMemberKind::ExternFunction(r#extern.preprocess(preprocessor)?),
              template_scope: None,
            },
          })
        },
        ast::Structure::Exported(_) => todo!("preprocess exported"),
        ast::Structure::TemplateScope(scope) => scope.preprocess(preprocessor)?,
        _ => None,
      }
    })
  }
}

impl Preprocess for ast::GlobalNamespace {
  type Out = Domain;

  fn preprocess(&self, preprocessor: &mut Preprocessor) -> Result<Self::Out, TypeCheckerError> {
    let mut inner = HashMap::new();

    for child in self.children.iter() {
      match child {
        ast::TopLevelStructure::Structure(struc) => {
          let Some(member) = struc.preprocess(preprocessor)? else {
            continue;
          };

          inner.insert(member.name, member.member);
        },
      };
    };

    Ok(Domain { inner })
  }
}
