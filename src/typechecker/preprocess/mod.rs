use std::collections::HashMap;

use std::rc::Rc;
use std::cell::RefCell;

use crate::tokenizer::GetSpan;
use crate::asterizer::ast;

use crate::typechecker::{
  check::Extends,
  Domain,
  DomainMember,
  error::*,
  NamedDomainMember,
  Preprocessor,
  TypeOf,
};

use crate::typechecker::lang::{
  Block,
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

  fn preprocess(&self, _preprocessor: &mut Preprocessor) -> Result<Self::Out, TypeCheckerError> {
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
        ast::Atom::Variable { .. } => todo!("preprocess variable"),
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
        Self::ControlFlow(_) => todo!(),
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
      arguments: VariableScope::from_vec(arguments),
      return_ty,
      body,
      span: self.span.to_owned(),
    })
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
            member: DomainMember::Function(
              func.preprocess(preprocessor)?
            ),
          })
        },
        ast::Structure::TypeAlias(alias) => {
          Some(NamedDomainMember {
            name: alias.name.to_owned(),
            member: DomainMember::Type(alias.ty.preprocess(preprocessor)?.into())
          })
        },
        ast::Structure::Interface(_) => todo!("preprocess interface"),
        ast::Structure::Struct(_) => todo!("preprocess struct"),
        ast::Structure::Class(_) => todo!("preprocess class"),
        ast::Structure::Extern(_) => todo!("preprocess extern"),
        ast::Structure::Exported(_) => todo!("preprocess exported"),
        ast::Structure::TemplateScope(_) => todo!("preprocess templatescope"),
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
