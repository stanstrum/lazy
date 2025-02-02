mod agent;
mod file;
mod translate;

mod lang;
mod token;

use std::collections::VecDeque;
use std::fmt::Debug;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender};
use std::thread::JoinHandle;
use std::time::Duration;

use agent::{Agent, AgentSignal};
use file::*;
use snafu::Whatever;

#[derive(Debug)]
enum CompilerSignal {
  RegisterFile { path: PathBuf },
  AgentError { id: usize, err: String },
  Finished { id: usize },
}

fn main() -> Result<(), ()> {
  // "snippets/00_basic.zy"
  let input_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("snippets/00_basic.zy");
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
    Box::new(agent::TranslateSignal(entry, entry_id)) as AgentSignal,
  ]);

  let mut errors = 0;

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

    // wait up to 500ms for a signal, otherwise consider it dead
    let result = compiler_rx.recv_timeout(Duration::from_millis(500)).ok();

    if result.is_some() {
      println!("[main] compiler signal: {:?}", result.as_ref().unwrap());
      agent_signal_count += 1;
    };

    match result {
      Some(CompilerSignal::Finished { id }) => {
        agents[id].as_mut().unwrap().free = true;
      },
      Some(CompilerSignal::RegisterFile { .. }) => todo!(),
      Some(CompilerSignal::AgentError { id, err }) => {
        println!("[main] received fatal error from thread #{id}: {err:?}");
        agents[id].as_mut().unwrap().free = true;
        errors += 1;
      },
      None => break,
    };

    if jobs.is_empty()
      && agents
        .iter()
        .all(|agent| agent.as_ref().map(|agent| agent.free).unwrap_or(true))
    {
      println!("[main] noticed we have no more work ... breaking");
      break;
    }
  }

  if agent_signal_count != 0 {
    println!("[main] processed {agent_signal_count} agent signals");
  }

  for (id, agent) in agents.into_iter().enumerate() {
    let Some(agent) = agent else {
      continue;
    };

    let _ = agent.tx.send(Box::new(agent::AgentDie));

    // agent never freed up before the compiler shut down
    if !agent.free {
      errors += 1;
      eprintln!("[main] [ERR] thread #{id} timed out");
    };

    // show this error after joining and only if we haven't seen the previous
    // timeout error
    if agent.handle.join().is_err() && agent.free {
      errors += 1;
      eprintln!("[main] [ERR] thread #{id} crashed");
    };
  }

  if errors == 0 {
    return Ok(());
  };

  eprintln!("[main] [ERR] encountered {errors} error(s) ... exiting");

  Err(())
}
