use crate::Result;
use crate::compiler::*;

macro_rules! make_todo_stage {
  ($name:ident: $trait:ident::<In = $in:ty>::$method:ident) => {
    make_todo_stage!(@inner: $name, $trait, $method, In = $in);
  };

  ($name:ident: $trait:ident::<In = $in:ty, Out = $out:ty>::$method:ident) => {
    make_todo_stage!(@inner: $name, $trait, $method, In = $in, Out = $out);
  };

  (@inner: $name:ident, $trait:ident, $method:ident, $($assoc:ident = $ty:ty),+) => {
    pub(super) struct $name;

    impl<W: CompilerWorkflow> $trait<W> for $name {
      $(type $assoc = $ty;)+

      fn new() -> Self { Self }
      fn $method(self, _: &mut Compiler<W>, _: Self::In) -> Result  {
        todo!()
      }
    }
  };
}

// These allow for making skeletons for the compiler workflow without fully
// implementing each stage
make_todo_stage! { Translator: Translate::<In = (), Out = ()>::translate }
make_todo_stage! { Checker: Check::<In = (), Out = ()>::check }
make_todo_stage! { Generator: Generate::<In = (), Out = ()>::generate }
make_todo_stage! { Outputter: Output::<In = ()>::output }
