mod impls;

use std::{cell::RefCell, rc::Rc};

pub use impls::*;
use super::*;

pub trait Task {
  fn explain(&self, lazy: &Lazy) -> String;
  fn execute(self: Box<Self>, lazy: &mut Lazy) -> Result<TaskResponse>;
}

pub enum TaskResponse {
  /// Pop this task -- it's done
  Pop,

  /// Replace this task -- its function/requirements/parameters have changed
  Replace(Box<dyn Task>),
}

pub struct Tasks {
  tasks: Vec<Box<dyn Task>>,
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
      tasks: Vec::new(),
      trace: Rc::new(RefCell::new(Vec::new())),
    }
  }

  /// Returns a boolean corresponding to whether any tasks were executed
  pub fn execute_pass(&mut self, lazy: &mut Lazy) -> Result<bool> {
    if self.tasks.is_empty() {
      return Ok(false);
    };

    let taken = self.tasks.drain(..).collect::<Vec<_>>();

    for task in taken {
      println!("execute task: {}", task.explain(lazy));
      let response = task.execute(lazy)?;

      match response {
        TaskResponse::Pop => {
          // do nothing
        },
        TaskResponse::Replace(replace) => {
          self.tasks.push(replace);
        },
      };
    };

    Ok(true)
  }

  pub fn task_status(&self, description: String) -> TaskStatus {
    let trace = self.trace.clone();

    trace.borrow_mut().push(description);

    TaskStatus {
      trace: trace.clone(),
    }
  }

  pub fn push(&mut self, task: impl Task + 'static) {
    self.tasks.push(Box::new(task));
  }
}
