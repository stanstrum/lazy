mod impls;

use std::{cell::RefCell, collections::VecDeque, rc::Rc};

pub use impls::*;
use super::*;

pub trait Task {
  fn explain(&self, lazy: &Lazy) -> String;
  fn execute(self: Box<Self>, lazy: &mut Lazy, tasks: &mut Tasks) -> Result<TaskResponse>;
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
    if self.tasks.is_empty() {
      return Ok(false);
    };

    while let Some(task) = self.tasks.pop_front() {
      let description = task.explain(lazy);
      let status = self.task_work(description);

      // println!("execute task:\n{}", self.explain(2));
      let response = task.execute(lazy, self)?;

      match response {
        TaskResponse::Pop => {
          // do nothing
        },
        TaskResponse::Replace(replace) => {
          self.tasks.push_back(replace);
        },
      };

      drop(status);
    };

    Ok(true)
  }

  #[must_use = "task status is to be kept alive as long as the task is working"]
  pub fn task_work(&self, description: String) -> TaskStatus {
    let trace = self.trace.clone();

    trace.borrow_mut().push(description);

    TaskStatus {
      trace: trace.clone(),
    }
  }

  pub fn push(&mut self, task: impl Task + 'static, source: &'static str) {
    println!(line_dbg!("push from {}"), source);
    self.tasks.push_back(Box::new(task));
  }

  pub fn explain(&self, offset: usize) -> String {
    let mut out = String::new();

    for (count, explain) in self.trace.borrow().iter().enumerate() {
      let spaces = " ".repeat(offset) + &"|   ".repeat(count);

      for line in explain.split('\n') {
        out += &format!("{spaces}{line}\n");
      };
    };

    // out += "###";

    out
  }
}
