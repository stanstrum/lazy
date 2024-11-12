use ast::Identifier;

use super::*;

impl Intrinsic {
  pub(super) fn try_from_slice(name: &str) -> Option<Self> {
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

impl<'a, S: Scope<Part = Identifier<DefaultWorkflow>>> ParseScope<'a> for Type<S> where Type<S>: SearchIn<S> {
  type In = ast::Type<DefaultWorkflow>;
  type Scope = S;

  fn parse_scope(_translator: &mut Translator<DefaultWorkflow>, _compiler: &Compiler<DefaultWorkflow>, input: Self::In, parent: &Option<WeakCell<Self::Scope>>) -> Result<RcCell<Self>> {
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
              parent: parent.clone().unwrap().into(),
              kind,
            }));
          };
        };

        // Otherwise, this qualified is as of yet unresolved -- return it as
        // such
        Ok(new_rc_cell(Self::Reference(new_rc_cell(Reference::Unresolved(new_rc_cell(UnresolvedReference {
          context: parent.clone().unwrap().into(),
          span: qualified.span,
          implicit: qualified.implicit,
          parts: qualified.parts,
        }))))))
      },
    }
  }
}

impl SearchIn<Module> for Type<Module> {
  fn parent(&self) -> Option<WeakCell<Module>> {
    match self {
      Type::Intrinsic { parent, .. } => Some(parent.clone().unwrap()),
      Type::Reference(reference) => reference.parent(),
      Type::TypeOfExpression { weak } => todo!(),
      Type::UnresolvedInstrinsic(weak) => todo!(),

    }
  }

  fn search_in(scope: &Module, index: &<Module as Scope>::Index) -> Result<ScopeSearch<Self, Module>> {
    Ok(
      match ModuleChild::search_in(scope, index)? {
        ScopeSearch::Found(rc) => {
          match &*rc.upgrade().unwrap().try_borrow().unwrap() {
            ModuleChild::Module(rc) => ScopeSearch::Next(Rc::downgrade(rc)),
            ModuleChild::Type(rc) => ScopeSearch::Found(Rc::downgrade(&rc.borrow().ty)),
            _ => ScopeSearch::None,
          }
        },
        ScopeSearch::Next(rc) => ScopeSearch::Next(rc),
        ScopeSearch::None => ScopeSearch::None,
      }
    )
  }
}
