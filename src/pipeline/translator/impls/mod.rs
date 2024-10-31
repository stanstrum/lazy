use super::*;

use crate::compiler::workflow::DefaultWorkflow;
use crate::Result;

use crate::asterizer::ast;

impl Intrinsic {
  fn try_from_slice(name: &str) -> Option<Self> {
    match name {
      "void" => Some(Self::Void),
      "u8" => Some(Self::U8),
      "u16" => Some(Self::U16),
      "u32" => Some(Self::U32),
      "u64" => Some(Self::U64),
      "i8" => Some(Self::I8),
      "i16" => Some(Self::I16),
      "i32" => Some(Self::I32),
      "i64" => Some(Self::I64),
      "f16" => Some(Self::F16),
      "f32" => Some(Self::F32),
      "f64" => Some(Self::F64),
      _ => None,
    }
  }
}

impl Scope for Module {
  type Index = str;
}

impl Scope for Function {
  type Index = str;
}

impl<S: Scope> Scope for RcCell<S> {
  type Index = S::Index;
}

impl SearchIn<Module> for Function {
  fn parent(&self) -> Option<RcCell<Module>> {
    Some(self.parent.clone().unwrap())
  }

  fn search_in(scope: &Module, index: &<Module as Scope>::Index) -> Result<ScopeSearch<Self, Module>> {
    Ok(
      match ModuleChild::search_in(scope, index)? {
        ScopeSearch::Found(rc) => {
          match &*rc.borrow() {
            ModuleChild::Function(rc) => ScopeSearch::Found(rc.clone()),
            ModuleChild::Module(rc) => ScopeSearch::Next(rc.clone()),
          }
        },
        ScopeSearch::Next(rc) => ScopeSearch::Next(rc),
        ScopeSearch::None => ScopeSearch::None,
      }
    )
  }
}

impl SearchIn<Module> for Module {
  fn parent(&self) -> Option<RcCell<Module>> {
    self.parent.clone().unwrap()
  }

  fn search_in(scope: &Module, index: &<Module as Scope>::Index) -> Result<ScopeSearch<Self, Module>> {
    Ok(
      match ModuleChild::search_in(scope, index)? {
        ScopeSearch::Found(rc) => {
          match &*rc.borrow() {
            ModuleChild::Module(rc) => ScopeSearch::Next(rc.clone()),
            _ => ScopeSearch::None,
          }
        },
        ScopeSearch::Next(rc) => ScopeSearch::Next(rc),
        ScopeSearch::None => ScopeSearch::None,
      }
    )
  }
}

impl SearchIn<Module> for ModuleChild {
  fn parent(&self) -> Option<RcCell<Module>> {
    match self {
      ModuleChild::Function(rc) => rc.borrow().parent(),
      ModuleChild::Module(rc) => rc.borrow().parent(),
    }
  }

  fn search_in(scope: &Module, index: &<Module as Scope>::Index) -> Result<ScopeSearch<Self, Module>> {
    Ok(
      scope.children.iter().find_map(|child| (
        match &*child.borrow() {
          ModuleChild::Function(rc) => {
            let name_matches = rc.borrow().name.name == index;

            name_matches.then(|| ScopeSearch::Found(new_rc_cell(ModuleChild::Function(rc.clone()))))
          },
          ModuleChild::Module(rc) => (rc.borrow().name == index).then(|| ScopeSearch::Next(rc.clone())),
        }
      )).unwrap_or(ScopeSearch::None)
    )
  }
}

impl SearchIn<Function> for FunctionArgument {
  fn parent(&self) -> Option<RcCell<Function>> {
    Some(self.parent.clone().unwrap())
  }

  fn search_in(scope: &Function, index: &<Function as Scope>::Index) -> Result<ScopeSearch<Self, Function>> {
    Ok(
      scope.arguments.iter().find_map(|argument| (argument.borrow().name.name == index).then(|| ScopeSearch::Found(argument.clone())))
      .unwrap_or(ScopeSearch::None)
    )
  }
}

impl<V: SearchIn<S>, S: Scope> Reference<V, S> {
  fn parent(&self) -> Option<RcCell<S>> {
    match self {
      Reference::Resolved(rc) => rc.borrow().parent(),
      Reference::Unresolved(rc) => Some(rc.borrow().context.clone().unwrap()),
    }
  }
}

impl SearchIn<Module> for Type<Module> {
  fn parent(&self) -> Option<RcCell<Module>> {
    match self {
      Type::Intrinsic { parent, .. } => Some(parent.clone().unwrap()),
      Type::Reference(reference) => reference.parent(),
    }
  }

  fn search_in(scope: &Module, index: &<Module as Scope>::Index) -> Result<ScopeSearch<Self, Module>> {
    Ok(
      match ModuleChild::search_in(scope, index)? {
        ScopeSearch::Found(rc) => {
          match &*rc.borrow() {
            ModuleChild::Module(rc) => ScopeSearch::Next(rc.clone()),
            _ => ScopeSearch::None,
          }
        },
        ScopeSearch::Next(rc) => ScopeSearch::Next(rc),
        ScopeSearch::None => ScopeSearch::None,
      }
    )
  }
}

impl<'a, S: Scope> ParseScope<'a> for Type<S> where Type<S>: SearchIn<S> {
  type In = ast::Type<DefaultWorkflow>;
  type Scope = S;

  fn parse_scope(_translator: &mut Translator<DefaultWorkflow>, input: Self::In, parent: &Option<RcCell<Self::Scope>>) -> Result<RcCell<Self>> {
    match input {
      ast::Type::Qualified(qualified) => {
        // If this qualified is not explicit and only has one part, it might be an
        // intrinsic type
        if !qualified.implicit && qualified.parts.len() == 1 {
          // We know from the above check that there is exactly one element in this
          // list.  Take that one.
          let argument = qualified.parts.first().unwrap();
          // Get the actual text of the identifier
          let name = &argument.name;

          // Check if this identifier corresponds to any Instrinsic
          if let Some(kind) = Intrinsic::try_from_slice(name) {
            // This means that this Type can be resolved as follows
            return Ok(new_rc_cell(Type::Intrinsic {
              parent: parent.as_ref().cloned().unwrap().into() ,
              kind,
            }));
          };
        };

        // Otherwise, this qualified is as of yet unresolved -- return it as
        // such
        Ok(Self::new_unknown(parent.as_ref().unwrap(), qualified))
      },
    }
  }
}

pub(crate) trait Something<S: Scope, T: SearchIn<S>> {
  fn scope_parent(&self) -> Option<RcCell<S>>;
}

impl<S: Scope, T: SearchIn<S>> Something<S, T> for RcCell<T> {
  fn scope_parent(&self) -> Option<RcCell<S>> {
    self.borrow().parent().clone()
  }
}

impl<'a> ParseScope<'a> for FunctionArgument {
  type In = ast::FunctionArgument<DefaultWorkflow>;
  type Scope = Function;

  fn parse_scope(translator: &mut Translator<DefaultWorkflow>, input: Self::In, parent: &Option<RcCell<Self::Scope>>) -> Result<RcCell<Self>> {
    let module = parent.as_ref().unwrap().scope_parent();

    let ty = translator.parse_scope::<Type<Module>, Module>(input.ty, &module)?;

    Ok(new_rc_cell(Self {
      name: input.identifier,
      ty,
      parent: parent.as_ref().cloned().unwrap().into(),
    }))
  }
}

impl<'a> ParseScope<'a> for Function {
  type In = ast::Function<DefaultWorkflow>;
  type Scope = Module;

  fn parse_scope(translator: &mut Translator<DefaultWorkflow>, input: Self::In, parent: &Option<RcCell<Self::Scope>>) -> Result<RcCell<Self>> {
    let rc = new_rc_cell(Self {
      parent: parent.as_ref().unwrap().clone().into(),
      name: input.identifier,
      arguments: vec![],
      return_ty: new_rc_cell(Type::Intrinsic {
        kind: Intrinsic::Void,
        parent: parent.as_ref().cloned().unwrap().into(),
      }),
    });

    let argument_parent = Some(rc.clone());

    let arguments = input.arguments
      .map(|x| x.arguments)
      .unwrap_or_default()
      .into_iter()
      .map(|argument| translator.parse_scope::<FunctionArgument, Function>(argument, &argument_parent))
      .collect::<Result<_>>()?;

    {
      let mut function = rc.borrow_mut();

      if let Some(input) = input.return_ty {
        function.return_ty = translator.parse_scope::<Type<Module>, Module>(input, parent)?;
      };

      function.arguments = arguments;
    };

    Ok(rc)
  }
}

impl<'a> ParseScope<'a> for Module {
  type In = ast::Namespace<DefaultWorkflow>;
  type Scope = Module;

  fn parse_scope(translator: &mut Translator<DefaultWorkflow>, input: Self::In, parent: &Option<RcCell<Self::Scope>>) -> Result<RcCell<Self>> {
    let module = new_rc_cell(Self {
      parent: parent.clone().into(),
      name: ModuleName::Identifier(input.identifier),
      children: vec![],
      span: input.span,
    });

    let child_parent = Some(module.clone());

    let children = input.children.into_iter()
      .map(|child| translator.parse_scope::<ModuleChild, Self::Scope>(child, &child_parent))
      .collect::<Result<_>>()?;

    {
      module.borrow_mut().children = children;
    };

    Ok(module)
  }
}

impl<'a> ParseScope<'a> for ModuleChild {
  type In = ast::NamespaceChild<DefaultWorkflow>;
  type Scope = Module;

  fn parse_scope(translator: &mut Translator<DefaultWorkflow>, input: Self::In, parent: &Option<RcCell<Self::Scope>>) -> Result<RcCell<Self>> {
    match input {
      ast::NamespaceChild::Namespace(namespace) => {
        let module  = translator.parse_scope(*namespace, parent)?;

        Ok(new_rc_cell(Self::Module(module)))
      },
      ast::NamespaceChild::Function(function) => {
        let function  = translator.parse_scope(function, parent)?;

        Ok(new_rc_cell(Self::Function(function)))
      },
    }
  }
}
