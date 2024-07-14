use crate::tokenizer;

use crate::typechecker::lang::{
  intrinsics::Intrinsic,
  Instruction,
  Type,
  TypeCell,
  Value,
};

pub(in crate::typechecker) trait PrettyPrint {
  fn pretty_print(&self) -> String;
}

impl PrettyPrint for Instruction {
  fn pretty_print(&self) -> String {
    match self {
      Instruction::Assign { .. } => todo!(),
      Instruction::Call { .. } => todo!(),
      Instruction::Return { .. } => todo!(),
      Instruction::Value(value) => value.pretty_print(),
      Instruction::Block(_) => todo!(),
    }
  }
}

impl PrettyPrint for tokenizer::Literal {
  fn pretty_print(&self) -> String {
    match &self.kind {
      tokenizer::LiteralKind::Integer(integer) => format!("{integer}"),
      tokenizer::LiteralKind::FloatingPoint(float) => format!("{float}"),
      tokenizer::LiteralKind::UnicodeString(string) => format!("\"{string}\""),
      tokenizer::LiteralKind::CString(string) => format!("c\"{string}\""),
      tokenizer::LiteralKind::ByteString(string) => format!("b\"{string}\""),
      tokenizer::LiteralKind::UnicodeChar(ch) => format!("'{ch}'"),
      tokenizer::LiteralKind::ByteChar(ch) => format!("b'{ch}'"),
    }
  }
}

impl PrettyPrint for Value {
  fn pretty_print(&self) -> String {
    match self {
      Value::Variable(_) => todo!(),
      Value::Instruction(instruction) => instruction.pretty_print(),
      Value::Literal { literal, .. } => literal.pretty_print(),
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
      Type::Intrinsic { kind, .. } => kind.pretty_print(),
      Type::Unresolved { implied, reference, template, .. } => {
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
      Type::UnsizedArrayOf { ty, .. } => {
        format!("[]{}", ty.pretty_print())
      },
      Type::SizedArrayOf { count, ty, .. } => {
        format!("[{}]{}", count.pretty_print(), ty.pretty_print())
      },
      Type::ReferenceTo { r#mut, ty, .. } => {
        if *r#mut {
          format!("&mut {}", ty.pretty_print())
        } else {
          format!("&{}", ty.pretty_print())
        }
      },
      Type::Shared(shared) => shared.pretty_print(),
      Type::Function { .. } => todo!(),
      Type::Struct { .. } => todo!(),
      Type::FuzzyInteger { .. } => "{integer}".to_string(),
      Type::FuzzyString { size, element_ty, .. } => format!("{{string: [{}]{}}}", *size, element_ty.pretty_print()),
      Type::Unknown { .. } => "{unknown}".to_string(),
    }
  }
}
