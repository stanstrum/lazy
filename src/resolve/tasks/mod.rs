mod impls;

use std::rc::Rc;
use std::collections::VecDeque;
use std::cell::RefCell;

pub use impls::*;

use super::*;

pub trait Task {
  fn explain(&self, lazy: &Lazy) -> String;
  fn execute(self: Box<Self>, lazy: &mut Lazy, tasks: &mut Tasks) -> Result<TaskResponse>;
}

impl Task for Box<dyn Task> {
  fn explain(&self, lazy: &Lazy) -> String {
    self.as_ref().explain(lazy)
  }

  fn execute(self: Box<Self>, lazy: &mut Lazy, tasks: &mut Tasks) -> Result<TaskResponse> {
    (*self).execute(lazy, tasks)
  }
}

pub enum TaskResponse {
  /// Pop this task -- it's done
  Pop,

  /// Replace this task -- its function/requirements/parameters have changed
  Replace(Box<dyn Task>),
}

pub struct Tasks {
  tasks: VecDeque<Box<dyn Task>>,
  trace: Rc<RefCell<Vec<String>>>,
}

pub struct TaskStatus {
  trace: Rc<RefCell<Vec<String>>>,
}

impl Drop for TaskStatus {
  fn drop(&mut self) {
    self.trace.borrow_mut().pop();
  }
}

impl Tasks {
  pub fn new() -> Self {
    Self {
      tasks: VecDeque::new(),
      trace: Rc::new(RefCell::new(Vec::new())),
    }
  }

  /// Returns a boolean corresponding to whether any tasks were executed
  pub fn execute_pass(&mut self, lazy: &mut Lazy) -> Result<bool> {
    // #[cfg(debug_assertions)]
    // println!(line_dbg!("execute_pass start"));

    if self.tasks.is_empty() {
      return Ok(false);
    };

    let mut count = 0;

    while let Some(task) = self.tasks.pop_front() {
      let description = format!(
        "execute_pass: {count}/{total}:\n{explain}",
        explain = task.explain(lazy),
        total = self.tasks.len(),
      );

      let response = self.work(description, |tasks| task.execute(lazy, tasks))?;

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

  pub fn work<T>(&mut self, description: String, f: impl FnOnce(&mut Tasks) -> T) -> T {
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

  pub fn push(&mut self, task: impl Task + 'static, _source: &'static str) {
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

  pub fn seed_error<T>(&self, base: ErrorBase) -> Result<T> {
    let call_stack = format!("Call Stack:\n{}", self.explain(4));

    Err(Box::new(Error {
      base,
      call_stack,
    }))
  }
}
