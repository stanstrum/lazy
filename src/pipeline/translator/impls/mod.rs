use crate::Result;

use crate::compiler::{
  Compiler,
  CompilerWorkflow,
};

use crate::asterizer::ast;
use crate::translator::lang::*;

use super::Translator;

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

impl<W: CompilerWorkflow> Translator<W> {
  // fn make_qualified_type(&self, qualified: ast::Qualified<W>) -> Result<Type<W>> {
  //   // If this qualified is not explicit and only has one part, it might be an
  //   // intrinsic type
  //   if qualified.implicit == false && qualified.parts.len() == 1 {
  //     // We know from the above check that there is exactly one element in this
  //     // list.  Take that one.
  //     let argument = qualified.parts.first().unwrap();
  //     // Get the actual text of the identifier
  //     let name = &argument.name;

  //     // Check if this identifier corresponds to any Instrinsic
  //     if let Some(intrinsic) = Intrinsic::try_from_slice(name) {
  //       // This means that this Type can be resolved as follows
  //       return Ok(Type::Intrinsic(intrinsic));
  //     };
  //   };

  //   // Otherwise, we don't recognize this reference at all
  //   Ok(Type::Unresolved(UnresolvedType {
  //     module: self.handle,
  //     qualified,
  //   }))
  // }

  // fn make_type(&self, ty: ast::Type<W>) -> Result<Type<W>> {
  //   match ty {
  //     ast::Type::Qualified(qualified) => self.make_qualified_type(qualified),
  //   }
  // }

  // fn make_function_argument(&self, argument: ast::FunctionArgument<W>) -> Result<FunctionArgument<W>> {
  //   Ok(FunctionArgument {
  //     name: argument.identifier,
  //     ty: self.make_type(argument.ty)?,
  //   })
  // }

  // fn make_function(&self, _compiler: &mut Compiler<W>, function: ast::Function<W>) -> Result<Function<W>> {
  //   let return_ty = match function.return_ty {
  //     Some(ty) => self.make_type(ty)?,
  //     None => Type::Intrinsic(Intrinsic::Void),
  //   };

  //   let arguments = function.arguments
  //     .map(|ast| ast.arguments)
  //     .unwrap_or_default()
  //     .into_iter()
  //     .map(|argument| self.make_function_argument(argument))
  //     .collect::<Result<_>>()?;

  //   Ok(Function {
  //     name: function.identifier,
  //     arguments,
  //     return_ty
  //   })
  // }

  // fn make_module_child(&self, compiler: &mut Compiler<W>, child: ast::NamespaceChild<W>) -> Result<ModuleChild<W>> {
  //   match child {
  //     ast::NamespaceChild::Namespace(namespace) => {
  //       self.make_module(compiler, *namespace)
  //         .map(Box::new)
  //         .map(ModuleChild::Module)
  //     },
  //     ast::NamespaceChild::Function(function) => {
  //       self.make_function(compiler, function)
  //         .map(ModuleChild::Function)
  //     },
  //   }
  // }

  // fn make_module(&self, _compiler: &mut Compiler<W>, _namespace: ast::Namespace<W>) -> Result<Module<W>> {
  //   todo!()
  // }

  pub(super) fn make_top_level_namespace(&self, compiler: &mut Compiler<W>, top_level: ast::TopLevelNamespace<W>) -> Result<Module<W>> {
    todo!()
  //   let children = top_level.children.into_iter()
  //     .map(
  //       |child| self.make_module_child(compiler, child)
  //     )
  //     .collect::<Result<_>>()?;

  //   Ok(Module {
  //     children,
  //     span: top_level.span,
  //   })
  }
}
