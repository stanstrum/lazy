mod signals;
pub(super) use signals::*;

use super::*;

pub(super) struct Agent {
  pub tx: Sender<AgentSignal>,
  pub handle: JoinHandle<()>,
  pub free: bool,
}

pub(super) trait AgentDispatch: Send + Debug {
  fn run(self: Box<Self>, id: usize, tx: &Sender<CompilerSignal>);
}

pub(super) type AgentSignal = Box<dyn AgentDispatch>;

impl Agent {
  pub(super) fn new(id: usize, agent_tx: &Sender<CompilerSignal>) -> Self {
    println!("[main] starting worker thread #{id}");

    let (compiler_tx, agent_rx) = channel::<AgentSignal>();
    let agent_tx = agent_tx.clone();

    let handle = std::thread::spawn(move || {
      println!("[thread #{id}] thread started");

      for job in agent_rx.iter() {
        println!("[thread #{id}] Job: {job:#?}");

        job.run(id, &agent_tx);
      }

      agent_tx.send(CompilerSignal::Finished { id }).unwrap();
    });

    Self {
      free: true,
      tx: compiler_tx,
      handle,
    }
  }
}
