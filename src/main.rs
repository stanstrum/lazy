mod store;
mod token;

use std::collections::VecDeque;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::thread::JoinHandle;
use std::time::Duration;

use store::*;

#[derive(Debug)]
enum CompilerSignal {
  Finished { id: usize },
}

#[derive(Debug)]
enum AgentSignal {
  Compile { entry_point: PathBuf },
}

struct Agent {
  tx: Sender<AgentSignal>,
  handle: JoinHandle<()>,
  free: bool,
}

impl Agent {
  fn new(id: usize, agent_tx: &Sender<CompilerSignal>) -> Self {
    let (compiler_tx, agent_rx) = channel::<AgentSignal>();

    let handle = std::thread::spawn(move || {
      let _ = id;
      let _ = agent_tx;

      println!("[thread #{id}] thread started");

      for job in agent_rx.iter() {
        println!("[thread #{id}] Job: {job:#?}");

        match job {
          _ => todo!(),
        }
      }
    });

    Self {
      free: true,
      tx: compiler_tx,
      handle,
    }
  }
}

fn main() {
  // "snippets/00_basic.zy"
  let input_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("snippets/10_read_source.zy");
  let output_path = std::env::current_dir().unwrap().join("a.out");

  let overwrite = false;
  let autorename = true;
  let thread_count = 2;

  let mut agents = (0..=thread_count)
    .map(|_| None::<Agent>)
    .collect::<Vec<_>>();

  let mut jobs = VecDeque::from([AgentSignal::Compile {
    entry_point: input_path,
  }]);

  let (agent_tx, compiler_rx) = channel::<CompilerSignal>();

  let mut agent_signal_count = 0;
  loop {
    for (id, agent) in agents.iter_mut().enumerate() {
      if agent.as_ref().is_some_and(|agent| !agent.free) {
        continue;
      };

      let Some(job) = jobs.pop_front() else {
        break;
      };

      let agent = agent.get_or_insert_with(|| Agent::new(id, &agent_tx));
      agent.free = false;
      agent.tx.send(job).unwrap();
    }

    match dbg!(compiler_rx.recv_timeout(Duration::from_millis(500)).ok()) {
      Some(CompilerSignal::Finished { id }) => {
        agent_signal_count += 1;
        agents[id].as_mut().unwrap().free = true;
      },
      None => break,
    };
  }

  if agent_signal_count != 0 {
    println!("[main] processed {agent_signal_count} agent signals");
  }

  for (id, agent) in agents.into_iter().enumerate() {
    let Some(agent) = agent else {
      continue;
    };

    if !agent.free {
      eprintln!("[main] [ERR] thread #{id} is not free; this usually means that it has crashed");
    };

    if agent.handle.join().is_err() {
      eprintln!(
        "[main] [ERR] thread #{id} couldn't be joined; this definitely means that it has crashed"
      );
    };
  }
}
