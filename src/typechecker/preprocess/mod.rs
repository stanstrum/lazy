use std::collections::HashMap;

use super::lang::{
  Block,
  Function,
  Instruction,
  Type,
  Value,
  Variable,
  VariableKind,
  VariableScope,
  VariableReference,
  intrinsics::Intrinsic,
};

use super::{
  Domain,
  DomainMember,
  NamedDomainMember,
  Preprocessor,
  TypeCheckerError,
};

use crate::asterizer::ast;
use crate::tokenizer::GetSpan;

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
          Instruction::Literal(literal.to_owned())
        },
        ast::Atom::StructInitializer(_) => todo!("preprocess structinitializer"),
        ast::Atom::Variable { .. } => todo!("preprocess variable"),
      }
    })
  }
}

impl Preprocess for ast::Expression {
  type Out = Instruction;

  fn preprocess(&self, preprocessor: &mut Preprocessor) -> Result<Self::Out, TypeCheckerError> {
    Ok({
      match self {
        ast::Expression::Atom(atom) => atom.preprocess(preprocessor)?,
        ast::Expression::Block(_) => todo!("preprocess block"),
        ast::Expression::SubExpression(_) => todo!("preprocess subexpression"),
        ast::Expression::Unary(_) => todo!("preprocess unary"),
        ast::Expression::Binary(_) => todo!("preprocess binary"),
      }
    })
  }
}

impl Preprocess for ast::Block {
  type Out = Block;

  fn preprocess(&self, preprocessor: &mut Preprocessor) -> Result<Self::Out, TypeCheckerError> {
    let mut variables = vec![];
    let mut body = vec![];

    let mut variable_map = HashMap::new();

    for child in self.children.iter() {
      match child {
        ast::BlockChild::Binding(binding) => {
          let variable_id = variables.len();

          variable_map.insert(&binding.name, variable_id);
          variables.push(Variable {
            kind: VariableKind::LocalVariable,
            ty: {
              if let Some(binding_type) = &binding.ty {
                binding_type.preprocess(preprocessor)?
              } else {
                Type::Unknown {
                  span: self.get_span().to_owned(),
                }
              }.into()
            },
            span: binding.span.to_owned(),
          });
        },
        _ => {},
      };
    };

    let scope = VariableScope::from_vec(variables);
    let inner = &scope.inner;

    let variable_scope =
      variable_map.into_iter().map(
        |(name, id)| {
          let owned_inner = inner.to_owned();
          let span = {
            owned_inner.borrow().get(id).unwrap().get_span().to_owned()
          };

          (
            name.to_owned(),
            VariableReference {
              scope: inner.to_owned(),
              id,
              span,
            }
          )
        });

    preprocessor.scope_stack.push(variable_scope.collect());

    for child in self.children.iter() {
      match child {
        ast::BlockChild::Binding(binding) => {
          if let Some(expr) = &binding.expr {
            let reference = preprocessor.find_variable_by_name(&binding.name, binding.get_span())?;

            let value = Value::Instruction(Box::new(
              expr.preprocess(preprocessor)?
            ));

            body.push(Instruction::Assign {
              dest: Value::Variable(reference),
              value,
              span: expr.get_span().to_owned(),
            });
          };
        },
        ast::BlockChild::Expression(expr) => {
          body.push(expr.preprocess(preprocessor)?);
        },
        ast::BlockChild::ControlFlow(_) => todo!(),
        ast::BlockChild::Return(_) => todo!(),
      };
    };

    if self.returns_last {
      let last = body.pop()
        .expect("a block that returns last must have at least one instruction");

      let span = last.get_span().to_owned();

      body.push(
        Instruction::Return {
          value: Value::Instruction(
            Box::new(last)
          ),
          span,
        }
      );
    };

    preprocessor.scope_stack.pop();

    Ok(Self::Out {
      // TODO: refactor here
      variables: scope,
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

    Ok(Function {
      arguments: VariableScope::from_vec(arguments),
      return_ty,
      body: self.body.preprocess(preprocessor)?,
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
