use super::{
  intrinsics::Intrinsic, Type, TypeCell, Value
};

pub(in crate::typechecker) trait PrettyPrint {
  fn pretty_print(&self) -> String;
}

impl PrettyPrint for Value {
  fn pretty_print(&self) -> String {
    match self {
      Value::Variable(_) => todo!(),
      Value::Instruction(_) => todo!(),
    }
  }
}

impl PrettyPrint for Intrinsic {
  fn pretty_print(&self) -> String {
    match self {
      Intrinsic::Void => "void",
      Intrinsic::U8 => "u8",
      Intrinsic::I8 => "i8",
      Intrinsic::U16 => "u16",
      Intrinsic::I16 => "i16",
      Intrinsic::U32 => "u32",
      Intrinsic::I32 => "i32",
      Intrinsic::U64 => "u64",
      Intrinsic::I64 => "i64",
      Intrinsic::F32 => "f32",
      Intrinsic::F64 => "f64",
    }.to_string()
  }
}

impl PrettyPrint for TypeCell {
  fn pretty_print(&self) -> String {
    self.borrow().pretty_print()
  }
}

impl PrettyPrint for Type {
  fn pretty_print(&self) -> String {
    match self {
      Type::Intrinsic(intrinsic) => intrinsic.pretty_print(),
      Type::Unresolved { implied, reference, template } => {
        let mut out = "{unresolved: ".to_string();

        if *implied {
          out += "::";
        };

        if let Some((last, parts)) = reference.inner.split_last() {
          for part in parts {
            out += part.as_str();
            out += "::";
          };

          out += last.as_str();
        };

        if let Some(template) = template.as_ref() {
          out += "<";

          if let Some((last_ty, tys)) = template.split_last() {
            for ty in tys {
              out += ty.pretty_print().as_str();
              out += ", ";
            };

            out += last_ty.pretty_print().as_str();
            out += ">";
          };
        };

        out += "}";

        out
      },
      Type::UnsizedArrayOf(ty) => {
        format!("[]{}", ty.pretty_print())
      },
      Type::SizedArrayOf { count, ty } => {
        format!("[{}]{}", count.pretty_print(), ty.pretty_print())
      },
      Type::ReferenceTo { r#mut, ty } => {
        if *r#mut {
          format!("&mut {}", ty.pretty_print())
        } else {
          format!("&{}", ty.pretty_print())
        }
      },
      Type::Shared(_) => todo!(),
      Type::Function { .. } => todo!(),
      Type::Struct(_) => todo!(),
      Type::FuzzyInteger => todo!(),
      Type::FuzzyString { size, element_ty } => format!("{{string: [{}]{}}}", *size, element_ty.pretty_print()),
      Type::Unknown => "{unknown}".to_string(),
    }
  }
}
