mod module;
mod traits;
pub(crate) mod error;

pub(crate) use module::CompilerModule;
pub(crate) use traits::*;
use error::*;

use std::path::PathBuf;
use std::marker::PhantomData;

pub(crate) type CompilerResult<T = ()> = Result<T, CompilerError>;

#[allow(unused)]
pub(crate) enum CompilerJob<W: CompilerWorkflow> {
  Taken,
  Unprocessed,
  Tokenized(<W::Tokenizer as Tokenize<W>>::Out),
  Asterized(<W::Asterizer as Asterize<W>>::Out),
  Translated(<W::Translator as Translate<W>>::Out),
  Checked(<W::Checker as Check<W>>::Out),
  Generated(<W::Generator as Generate<W>>::Out),
}

pub(crate) struct CompilerStore<W: CompilerWorkflow> {
  modules: Vec<CompilerModule<W>>,
  marker: PhantomData<W>,
}

#[derive(Clone, Copy)]
pub(crate) struct CompilerStoreHandle<W: CompilerWorkflow> {
  index: usize,
  marker: PhantomData<CompilerModule<W>>,
}

#[allow(unused)]
pub(super) struct CompilerSettings {
  pub(super) input_file: PathBuf,
  pub(super) output_file: PathBuf,
  pub(super) llc: PathBuf,
  pub(super) cc: PathBuf,
}

pub(super) struct Compiler<W: CompilerWorkflow> {
  pub(crate) settings: CompilerSettings,
  pub(crate) store: CompilerStore<W>,
}

impl<W: CompilerWorkflow> CompilerJob<W> {
  fn stage(&self) -> Option<CompilationStage> {
    match self {
      CompilerJob::Taken => None,
      CompilerJob::Unprocessed => Some(CompilationStage::Tokenize),
      CompilerJob::Tokenized(_) => Some(CompilationStage::Asterize),
      CompilerJob::Asterized(_) => Some(CompilationStage::Translate),
      CompilerJob::Translated(_) => Some(CompilationStage::Check),
      CompilerJob::Checked(_) => Some(CompilationStage::Generate),
      CompilerJob::Generated(_) => Some(CompilationStage::Output),
    }
  }
}

#[derive(PartialEq, PartialOrd)]
enum CompilationStage {
  Tokenize,
  Asterize,
  Translate,
  Check,
  Generate,
  Output,
  Done,
}

pub(crate) struct TakenCompilerModule<W: CompilerWorkflow> {
  pub(crate) handle: CompilerStoreHandle<W>,
  pub(crate) data: CompilerJob<W>,
}

trait JobStore<W: CompilerWorkflow> where Self: Sized {
  fn store_by_handle(self, store: &mut CompilerStore<W>, handle: CompilerStoreHandle<W>) -> CompilerStoreHandle<W>;
  fn store(self, store: &mut CompilerStore<W>) -> CompilerStoreHandle<W>;
}

impl<W: CompilerWorkflow> JobStore<W> for CompilerModule<W> {
  fn store_by_handle(self, store: &mut CompilerStore<W>, handle: CompilerStoreHandle<W>) -> CompilerStoreHandle<W> {
    store.modules.insert(handle.index, self);
    handle
  }

  fn store(self, store: &mut CompilerStore<W>) -> CompilerStoreHandle<W> {
    let handle = store.register_module(&self);
    self.store_by_handle(store, handle)
  }
}

impl<W: CompilerWorkflow> JobStore<W> for TakenCompilerModule<W> {
  fn store_by_handle(self, store: &mut CompilerStore<W>, handle: CompilerStoreHandle<W>) -> CompilerStoreHandle<W> {
    store.modules[handle.index].data = self.data;
    handle
  }

  fn store(self, store: &mut CompilerStore<W>) -> CompilerStoreHandle<W> {
    let handle = self.handle;
    self.store_by_handle(store, handle)
  }
}

impl<W: CompilerWorkflow> CompilerStore<W> {
  fn new() -> Self {
    Self {
      modules: vec![],
      marker: Default::default(),
    }
  }

  fn add_module(&mut self, module: CompilerModule<W>) -> CompilerStoreHandle<W> {
    let index = self.modules.len();

    self.modules.push(module);

    CompilerStoreHandle {
      index,
      marker: Default::default(),
    }
  }

  fn find_module(&self, module: &CompilerModule<W>) -> Option<CompilerStoreHandle<W>> {
    for (index, curr) in self.modules.iter().enumerate() {
      if module.is_same_path(curr) {
        return Some(CompilerStoreHandle {
          index,
          marker: Default::default(),
        });
      };
    };

    None
  }

  fn register_module(&mut self, module: &CompilerModule<W>) -> CompilerStoreHandle<W> {
    if let Some(handle) = self.find_module(module) {
      return handle;
    };

    self.add_module(CompilerModule {
      path: module.path.to_owned(),
      data: CompilerJob::Taken,
    })
  }

  fn store_module<T: JobStore<W>>(&mut self, module: T) -> CompilerStoreHandle<W> {
    module.store(self)
  }

  pub(crate) fn get_module(&self, handle: &CompilerStoreHandle<W>) -> &CompilerModule<W> {
    &self.modules[handle.index]
  }

  fn get_module_mut(&mut self, handle: &CompilerStoreHandle<W>) -> &mut CompilerModule<W> {
    &mut self.modules[handle.index]
  }

  fn take_module(&mut self, handle: &CompilerStoreHandle<W>) -> TakenCompilerModule<W>  {
    let module = self.get_module_mut(handle);

    let mut taken = TakenCompilerModule::<W> {
      handle: <CompilerStoreHandle<W> as Clone>::clone(handle),
      data: CompilerJob::Taken::<W>,
    };

    std::mem::swap(&mut module.data, &mut taken.data);

    taken
  }
}

impl<W: CompilerWorkflow> Compiler<W> {
  pub(crate) fn new(settings: CompilerSettings) -> Self {
    Self {
      settings,
      store: CompilerStore::new(),
    }
  }

  fn bring_to_stage(&mut self, handle: &CompilerStoreHandle<W>, stage: CompilationStage) -> CompilerResult<()> {
    while {
      let module = self.store.get_module(handle);

      let Some(module_stage) = module.data.stage() else {
        warn!("module {} (id {}): no stage", module.path.to_string_lossy(), handle.index);
        return Ok(());
      };

      assert!(module_stage <= stage);
      module_stage < stage
    } {
      let mut module = self.store.take_module(handle);
      let log_prefix = || format!("module {:?} (id #{})", &self.store.get_module(handle).path, handle.index);

      match module.data {
        CompilerJob::Taken => {
          warn!("{}: taken", log_prefix());
          return Ok(());
        },
        CompilerJob::Unprocessed => {
          trace!("{}: tokenize", log_prefix());
          let tokenized = W::Tokenizer::new().tokenize(self, TakenCompilerModule {
            handle: *handle,
            data: module.data,
          })?;
          module.data = CompilerJob::Tokenized(tokenized);
        },
        CompilerJob::Tokenized(input) => {
          trace!("{}: asterize", log_prefix());
          let asterized = W::Asterizer::new().asterize(self, input)?;
          module.data = CompilerJob::Asterized(asterized);
        },
        CompilerJob::Asterized(input) => {
          trace!("{}: translate", log_prefix());
          let translated = W::Translator::new().translate(self, input)?;
          module.data = CompilerJob::Translated(translated);
        },
        CompilerJob::Translated(input) => {
          trace!("{}: check", log_prefix());
          let checked = W::Checker::new().check(self, input)?;
          module.data = CompilerJob::Checked(checked);
        },
        CompilerJob::Checked(input) => {
          trace!("{}: generate", log_prefix());
          let generated = W::Generator::new().generate(self, input)?;
          module.data = CompilerJob::Generated(generated);
        },
        CompilerJob::Generated(input) => {
          trace!("{}: output", log_prefix());
          W::Outputter::new().output(self, input)?;

          return Ok(());
        },
      };

      self.store.store_module(module);
    };

    todo!()
  }

  pub(crate) fn compile(&mut self) -> CompilerResult<<W::Generator as Generate<W>>::Out> {
    let module: CompilerModule<W> = self.settings.input_file.as_path().try_into()?;
    let handle = self.store.store_module(module);

    self.bring_to_stage(&handle, CompilationStage::Done)?;

    todo!()
  }
}
