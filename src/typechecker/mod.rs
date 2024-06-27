mod error;
mod r#type;

// use std::path::Path;
use std::collections::HashMap;

use crate:: compiler::{
  Compiler,
  Handle,
  SourceFile,
  SourceFileData,
};

use crate::asterizer::ast;
use crate::CompilationError;

pub(crate) use error::*;

use r#type::Type;

#[allow(unused)]
#[derive(Debug)]
struct DomainReference {
  handle: Handle,
  inner: Vec<String>,
}

impl DomainReference {
  fn new(handle: Handle) -> Self {
    Self {
      inner: vec![],
      handle,
    }
  }
}

#[derive(Debug)]
enum Instruction {}

#[allow(unused)]
#[derive(Debug)]
struct Variable {
  ty: Type,
}

#[allow(unused)]
#[derive(Debug)]
struct Block {
  variables: Vec<Variable>,
  body: Vec<Instruction>,
}

impl Block {
  fn new() -> Self {
    Self {
      variables: vec![],
      body: vec![],
    }
  }
}

#[allow(unused)]
#[derive(Debug)]
struct Function {
  arguments: Vec<Type>,
  return_ty: Type,
  body: Block,
}

#[allow(unused)]
#[derive(Debug)]
enum DomainMember {
  Domain(Domain),
  Function(Function),
  Type(Type),
}

#[allow(unused)]
#[derive(Debug)]
struct Program {
  inner: HashMap<Handle, Domain>,
}

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct Domain {
  inner: HashMap<String, DomainMember>,
}

#[allow(unused)]
pub(crate) struct TypeChecker<'a> {
  compiler: &'a Compiler,
  reference: DomainReference,
  modules: Program,
}

trait Preprocess {
  type Out;

  fn preprocess(&self) -> Self::Out;
}

struct NamedDomainMember {
  name: String,
  member: DomainMember,
}

impl Preprocess for ast::Structure {
  type Out = Option<NamedDomainMember>;

  fn preprocess(&self) -> Self::Out {
    match self {
      ast::Structure::Namespace(_) => todo!("preprocess namespace"),
      ast::Structure::Function(ast::Function { decl, .. }) => {
        let mut arguments = vec![];

        if let Some(decl_args) = &decl.args {
          for arg in decl_args.args.iter() {
            arguments.push((&arg.ty).into());
          };
        };

        Some(NamedDomainMember {
          name: decl.name.to_owned(),
          member: DomainMember::Function(Function {
            arguments,
            return_ty: decl.return_type.as_ref().into(),
            body: Block::new(),
          })
        })
      },
      ast::Structure::TypeAlias(alias) => {
        Some(NamedDomainMember {
          name: alias.name.to_owned(),
          member: DomainMember::Type((&alias.ty).into())
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

  fn preprocess(&self) -> Self::Out {
    let mut inner = HashMap::new();

    for child in self.children.iter() {
      match child {
        ast::TopLevelStructure::Structure(struc) => {
          let Some(member) = struc.preprocess() else {
            continue;
          };

          inner.insert(member.name, member.member);
        },
      };
    };

    Domain { inner }
  }
}

impl<'a> TypeChecker<'a> {
  pub(crate) fn new(compiler: &'a Compiler) -> Self {
    Self {
      compiler,
      reference: DomainReference::new(compiler.entry_point),
      modules: Program { inner: HashMap::new() },
    }
  }

  pub(crate) fn preprocess(_compiler: &mut Compiler, file: SourceFile, _handle: &Handle) -> Result<SourceFile, CompilationError> {
    let SourceFile {
      path,
      data: SourceFileData::Asterized(ast),
      debug_info,
    } = file else {
      unreachable!();
    };

    let program = dbg!(ast.preprocess());

    Ok(SourceFile {
      path,
      data: SourceFileData::TypeChecked(program),
      debug_info,
    })
  }

  pub(crate) fn check(self) -> Result<(), TypeCheckerError> {
    todo!()
  }
}
