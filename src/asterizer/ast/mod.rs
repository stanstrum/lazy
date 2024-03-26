use typename::TypeName;

use crate::asterizer::{
  TokenStream,
  AsterizerError
};

#[macro_export]
macro_rules! import_export {
  ($name:ident) => {
    pub(self) mod $name;
    #[allow(unused)]
    pub(crate) use $name::*;
  };

  ($name:ident, $($names:ident,)+) => {
    import_export!($name);
    import_export!($($names),+);
  };

  ($($names:ident),+) => {
    import_export!($($names,)+);
  };
}

import_export! {
  global_namespace,
  structure,
  function,
  expression,
  ty,
}

pub(crate) trait MakeAst where Self: Sized + TypeName {
  fn make(stream: &mut TokenStream) -> Result<Option<Self>, AsterizerError>;
}
