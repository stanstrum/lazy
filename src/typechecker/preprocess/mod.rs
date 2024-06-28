use std::collections::HashMap;

use super::lang::{
  Block,
  Function,
  Instruction,
  Type,
  Variable,
  VariableKind,
  VariableScope,
};

use super::{
  Domain,
  DomainMember,
  DomainReference,
  NamedDomainMember,
};

use crate::asterizer::ast;

pub(super) trait Preprocess {
  type Out;

  fn preprocess(&self, reference: &DomainReference) -> Self::Out;
}

impl Preprocess for ast::Block {
  type Out = Block;

  fn preprocess(&self, reference: &DomainReference) -> Self::Out {
    let mut variables = vec![];
    let mut body: Vec<Instruction> = vec![];

    let mut variable_map = HashMap::new();

    for child in self.children.iter() {
      match child {
        ast::BlockChild::Expression(_) => todo!(),
        ast::BlockChild::Binding(binding) => {
          let variable_id = variables.len();

          variable_map.insert(&binding.name, variable_id);
          variables.push(Variable {
            kind: VariableKind::LocalVariable,
            ty: {
              if let Some(binding_type) = &binding.ty {
                Type::from_ast(&binding_type, reference)
              } else {
                Type::Unknown
              }
            }
          });

          if let Some(expr) = &binding.expr {
            // body.push(expr.p(reference));
            todo!()
          };
        },
        ast::BlockChild::ControlFlow(_) => todo!(),
        ast::BlockChild::Return(_) => todo!(),
      };
    };

    Self::Out {
      variables: VariableScope::from_vec(variables),
      body: todo!(),
    }
  }
}

impl Preprocess for ast::Function {
  type Out = Function;

  fn preprocess(&self, reference: &DomainReference) -> Self::Out {
    let mut arguments = vec![];

    if let Some(decl_args) = &self.decl.args {
      for arg in decl_args.args.iter() {
        arguments.push(Variable {
          kind: VariableKind::Argument,
          ty: Type::from_ast(&arg.ty, reference),
        });
      };
    };

    Function {
      arguments: VariableScope::from_vec(arguments),
      return_ty: Type::from_ast_optional(self.decl.return_type.as_ref(), reference),
      body: self.body.preprocess(reference),
    }
  }
}

impl Preprocess for ast::Structure {
  type Out = Option<NamedDomainMember>;

  fn preprocess(&self, reference: &DomainReference) -> Self::Out {
    match self {
      ast::Structure::Namespace(_) => todo!("preprocess namespace"),
      ast::Structure::Function(func) => {
        Some(NamedDomainMember {
          name: func.decl.name.to_owned(),
          member: DomainMember::Function(
            func.preprocess(reference)
          ),
        })
      },
      ast::Structure::TypeAlias(alias) => {
        Some(NamedDomainMember {
          name: alias.name.to_owned(),
          member: DomainMember::Type(Type::from_ast(&alias.ty, reference))
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
  }
}

impl Preprocess for ast::GlobalNamespace {
  type Out = Domain;

  fn preprocess(&self, reference: &DomainReference) -> Self::Out {
    let mut inner = HashMap::new();

    for child in self.children.iter() {
      match child {
        ast::TopLevelStructure::Structure(struc) => {
          let Some(member) = struc.preprocess(reference) else {
            continue;
          };

          inner.insert(member.name, member.member);
        },
      };
    };

    Domain { inner }
  }
}
