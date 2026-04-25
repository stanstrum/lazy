use lazy_macros::line_dbg;
use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use crate::{Compiler, error::{ResolveError, ResolveErrorBase}};

pub trait Task<C: Compiler> {
  fn explain(&self, store: &C::Store<'_>) -> String;
  fn execute(self: Box<Self>, store: &mut C::Store<'_>, tasks: &mut Tasks<C>) -> Result<TaskResponse<C>, Box<ResolveError<C>>>;
}

impl<C: Compiler> Task<C> for Box<dyn Task<C>> {
  fn explain(&self, store: &C::Store<'_>) -> String {
    self.as_ref().explain(store)
  }

  fn execute(self: Box<Self>, store: &mut C::Store<'_>, tasks: &mut Tasks<C>) -> Result<TaskResponse<C>, Box<ResolveError<C>>> {
    (*self).execute(store, tasks)
  }
}

pub enum TaskResponse<C: Compiler> {
  /// Pop this task -- it's done
  Pop,

  /// Replace this task -- its function/requirements/parameters have changed
  Replace(Box<dyn Task<C>>),
}

pub struct Tasks<C> {
  tasks: VecDeque<Box<dyn Task<C>>>,
  trace: Rc<RefCell<Vec<String>>>,
}

impl<C: Compiler + 'static> Tasks<C> {
  pub fn new() -> Self {
    Self {
      tasks: VecDeque::new(),
      trace: Rc::new(RefCell::new(Vec::new())),
    }
  }

  /// Returns a boolean corresponding to whether any tasks were executed
  pub fn execute_pass(&mut self, store: &mut C::Store<'_>) -> Result<bool, Box<ResolveError<C>>> {
    // #[cfg(debug_assertions)]
    // println!(line_dbg!("execute_pass start"));

    if self.tasks.is_empty() {
      return Ok(false);
    };

    let mut count = 0;

    while let Some(task) = self.tasks.pop_front() {
      let description = format!(
        "execute_pass: {count}/{total}:\n{explain}",
        explain = task.explain(store),
        total = self.tasks.len(),
      );

      let response = self.work(description, |tasks| task.execute(store, tasks))?;

      match response {
        TaskResponse::Pop => {
          // do nothing
        },
        TaskResponse::Replace(replace) => {
          self.push(replace, line_dbg!("here"));
        },
      };

      count += 1;
    };

    Ok(true)
  }

  pub fn work<T>(&mut self, description: String, f: impl FnOnce(&mut Tasks<C>) -> T) -> T {
    #[cfg(debug_assertions)]
    // Push the description to the stack
    self.trace.borrow_mut().push(description);

    // #[cfg(debug_assertions)]
    // // SPONGE: Print the explain() message for the whole stack
    // println!("{}", self.explain(0));

    // Run the task
    let result = f(self);

    #[cfg(debug_assertions)]
    // Drop the handle
    self.trace.borrow_mut().pop()
      .expect("to pop status from trace");

    // Return the result, error or not
    result
  }

  pub fn push(&mut self, task: impl Task<C> + 'static, _source: &'static str) {
    // #[cfg(debug_assertions)] println!(line_dbg!("push from {}"), _source);
    self.tasks.push_back(Box::new(task));
  }

  pub fn explain(&self, offset: usize) -> String {
    let mut out = String::new();

    for (count, explain) in self.trace.borrow().iter().enumerate() {
      let spaces = " ".repeat(offset) + &"|  ".repeat(count);

      for line in explain.split('\n') {
        out += &format!("{spaces}{line}\n");
      };
    };

    // out += "###";

    out
  }

  pub fn seed_error<T>(&self, base: ResolveErrorBase<C>) -> Result<T, Box<ResolveError<C>>> {
    let call_stack = format!("Call Stack:\n{}", self.explain(4));

    Err(Box::new(ResolveError {
      base,
      call_stack,
    }))
  }
}
