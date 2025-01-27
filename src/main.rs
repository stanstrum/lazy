mod store;
mod token;

use std::clone;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender};
use std::thread::JoinHandle;
use std::time::Duration;

use snafu::Whatever;
use store::*;

#[derive(Debug)]
enum CompilerSignal {
  RegisterFile { path: PathBuf },
  Finished { id: usize },
}

trait AgentDispatch: Send + Debug {
  type Out;

  fn run(self: Box<Self>, id: usize, tx: &Sender<CompilerSignal>) -> Self::Out;
}

type AgentSignal<Out = ()> = Box<dyn AgentDispatch<Out = Out>>;

#[derive(Debug)]
struct Translate(LazyFile);

impl AgentDispatch for Translate {
  type Out = ();

  fn run(self: Box<Self>, id: usize, tx: &Sender<CompilerSignal>) -> Self::Out {
    let Translate(file) = *self;
    let tokens: Vec<_> = token::tokenize(&file).unwrap().collect();
    dbg!(tokens);

    todo!();
  }
}

struct Agent {
  tx: Sender<AgentSignal>,
  handle: JoinHandle<()>,
  free: bool,
}

impl Agent {
  fn new(id: usize, agent_tx: &Sender<CompilerSignal>) -> Self {
    println!("[main] starting worker thread #{id}");

    let (compiler_tx, agent_rx) = channel::<AgentSignal>();
    let agent_tx = agent_tx.clone();

    let handle = std::thread::spawn(move || {
      println!("[thread #{id}] thread started");

      for job in agent_rx.iter() {
        println!("[thread #{id}] Job: {job:#?}");

        job.run(id, &agent_tx);
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

  let mut files: Vec<LazyFile> = vec![];
  let mut register_file = |mut file: LazyFile| -> Result<usize, Whatever> {
    file.solidify()?;

    for (id, existing_file) in files.iter().enumerate() {
      if &file == existing_file {
        return Ok(id);
      };
    }

    files.push(file);
    Ok(files.len() - 1)
  };

  let entry_id = register_file(LazyFile::new(input_path)).unwrap();
  let entry = files[entry_id].clone();

  let mut jobs: VecDeque<AgentSignal> = VecDeque::from([
    // --
    Box::new(Translate(entry)) as AgentSignal,
  ]);

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
      Some(CompilerSignal::RegisterFile { .. }) => todo!(),
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
      eprintln!("[main] [ERR] thread #{id} is hung");
    };

    if agent.handle.join().is_err() {
      eprintln!("[main] [ERR] thread #{id} crashed");
    };
  }
}
