use super::*;

pub struct Resolver<'store, 'pool, 'tasks, C: Compiler> {
  pub(crate) store: &'store mut C::Store<'pool>,
  pub(crate) tasks: &'tasks Tasks<C>,
  pub(crate) global: C::ModuleReference,
}

impl<'store, 'pool, 'tasks, C: Compiler> Resolver<'store, 'pool, 'tasks, C> {
  pub(crate) fn new(store: &'store mut C::Store<'pool>, global: C::ModuleReference, tasks: &'tasks Tasks<C>) -> Result<C, Self> {
    Ok(Self {
      store,
      tasks,
      global,
    })
  }

  /// SPONGE: I don't think this method is doing what I thought it'd do
  pub(crate) fn resolve_tasks(&mut self, _description: String) -> Result<C> {
    self.execute_pass().and(Ok(()))
  }

  pub(crate) fn work<T>(&mut self, description: String, cb: impl FnOnce(&mut Self) -> T) -> T {
    #[cfg(debug_assertions)]
    // Push the description to the stack
    self.tasks.trace.borrow_mut().push(description);

    // #[cfg(debug_assertions)]
    // // SPONGE: Print the explain() message for the whole stack
    // println!("{}", self.explain(0));

    // Run the task
    let result = cb(self);

    #[cfg(debug_assertions)]
    // Drop the handle
    self.tasks.trace.borrow_mut().pop()
      .expect("to pop status from trace");

    // Return the result, error or not
    result
  }

  /// Returns a boolean corresponding to whether any tasks were executed
  pub(crate) fn execute_pass(&mut self) -> Result<C, bool> {
    // #[cfg(debug_assertions)]
    // println!(line_dbg!("execute_pass start"));

    let total = self.tasks.tasks.borrow().len();

    if total == 0 {
      return Ok(false);
    };

    let mut count = 0;

    loop {
      let Some(task) = ({
        let mut borrow = self.tasks.tasks.borrow_mut();
        let value = borrow.pop_front();
        drop(borrow);
        value
      }) else {
        break;
      };

      let description = {
        let explain = task.explain(self);

        format!(
          "execute_pass: {count}/{total}:\n{explain}",
        ).to_owned()
      };

      let response = self.work(description, |resolver| task.execute(resolver))?;

      match response {
        TaskResponse::Pop => {
          // do nothing
        },
        TaskResponse::Replace(replace) => {
          self.tasks.tasks.borrow_mut().push_back(replace);
        },
      };

      count += 1;
    };

    Ok(true)
  }

  fn explain(&self, offset: usize) -> String {
    let mut out = String::new();

    for (count, explain) in self.tasks.trace.borrow().iter().enumerate() {
      let spaces = " ".repeat(offset) + &"|  ".repeat(count);

      for line in explain.split('\n') {
        out += &format!("{spaces}{line}\n");
      };
    };

    // out += "###";

    out
  }

  pub(crate) fn seed_error<T>(&self, base: ResolveErrorBase<C>) -> Result<C, T> {
    let call_stack = format!("Call Stack:\n{}", self.explain(4));

    Err(Box::new(ResolveError {
      base,
      call_stack,
    }))
  }
}
