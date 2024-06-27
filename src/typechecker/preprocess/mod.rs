use std::collections::HashMap;

use super::{
  Domain,
  DomainReference,
  DomainMember,
  NamedDomainMember,
  lang::{
    Type,
    Function,
    Block,
  },
};
use crate::asterizer::ast;

pub(super) trait Preprocess {
  type Out;

  fn preprocess(&self, reference: &DomainReference) -> Self::Out;
}

impl Preprocess for ast::Structure {
  type Out = Option<NamedDomainMember>;

  fn preprocess(&self, reference: &DomainReference) -> Self::Out {
    match self {
      ast::Structure::Namespace(_) => todo!("preprocess namespace"),
      ast::Structure::Function(ast::Function { decl, body: ast_body }) => {
        let mut arguments = vec![];

        if let Some(decl_args) = &decl.args {
          for arg in decl_args.args.iter() {
            arguments.push(Type::from_ast(&arg.ty, reference));
          };
        };

        for expr in ast_body.children.iter() {
          match expr {
            ast::BlockChild::Expression(_) => todo!(),
            ast::BlockChild::Binding(_) => todo!(),
            ast::BlockChild::ControlFlow(_) => todo!(),
            ast::BlockChild::Return(_) => todo!(),
          };
        };

        Some(NamedDomainMember {
          name: decl.name.to_owned(),
          member: DomainMember::Function(Function {
            arguments,
            return_ty: Type::from_ast_optional(decl.return_type.as_ref(), reference),
            body: Block::new(),
          })
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
