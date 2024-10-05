use std::path::{
  Path,
  PathBuf,
};

#[derive(Clone)]
pub(crate) struct EntryModule {
  path: PathBuf,
}

impl EntryModule {
  pub(super) fn read(self) -> Result<std::io::BufReader<std::fs::File>, String> {
    Ok(
      std::io::BufReader::new(
        std::fs::File::open(self.path)
          .map_err(|err| err.to_string())?
      )
    )
  }
}

impl TryFrom<&Path> for EntryModule {
  type Error = String;

  fn try_from(path: &Path) -> Result<Self, Self::Error> {
    if !path.exists() {
      Err(format!("input file does not exist: {}", path.to_string_lossy()))
    } else if path.is_dir() {
      let mut path = path.to_owned();
      path.push("index.zy");

      if path.is_dir() {
        return Err(format!("input file may not be a directory: {}", path.to_string_lossy()));
      };

      EntryModule::try_from(path.as_path())
    } else {
      Ok(Self {
        path: path.to_path_buf(),
      })
    }
  }
}

pub(super) struct CompilerSettings {
  pub(super) input_file: EntryModule,
  pub(super) output_file: PathBuf,
  pub(super) llc: PathBuf,
  pub(super) cc: PathBuf,
}

type TokenizerIn = EntryModule;
pub(super) struct Compiler<
  'a,
  TokenizerOut,
  AsterizerOut,
  TranslatorOut,
  CheckerOut,
  GeneratorOut,
  OutputterOut,
> {
  settings: CompilerSettings,
  tokenizer: WorkflowStep<'a, TokenizerIn, Self>,
  asterizer: WorkflowStep<'a, TokenizerOut, Self>,
  translator: WorkflowStep<'a, AsterizerOut, Self>,
  checker: WorkflowStep<'a, TranslatorOut, Self>,
  generator: WorkflowStep<'a, CheckerOut, Self>,
  outputter: WorkflowStep<'a, GeneratorOut, Self>,
  jobs: Vec<CompilerJob<
    TokenizerOut,
    AsterizerOut,
    TranslatorOut,
    CheckerOut,
    GeneratorOut,
  >>,
}

type WorkflowStep<'a, In, Compiler> = fn(compiler: &'a mut Compiler, value: In) -> Result<&'a mut Compiler, String>;

enum CompilerJob<
  TokenizerOut,
  AsterizerOut,
  TranslatorOut,
  CheckerOut,
  GeneratorOut,
> {
  Unprocessed(TokenizerIn),
  Tokenized(TokenizerOut),
  Asterized(AsterizerOut),
  Translated(TranslatorOut),
  Checked(CheckerOut),
  Generated(GeneratorOut),
}

impl<
  'a,
  TokenizerOut,
  AsterizerOut,
  TranslatorOut,
  CheckerOut,
  GeneratorOut,
  OutputterOut,
> Compiler<
  'a,
  TokenizerOut,
  AsterizerOut,
  TranslatorOut,
  CheckerOut,
  GeneratorOut,
  OutputterOut,
> {
  pub(crate) fn new(
    settings: CompilerSettings,
    tokenizer: WorkflowStep<'a, TokenizerIn, Self>,
    asterizer: WorkflowStep<'a, TokenizerOut, Self>,
    translator: WorkflowStep<'a, AsterizerOut, Self>,
    checker: WorkflowStep<'a, TranslatorOut, Self>,
    generator: WorkflowStep<'a, CheckerOut, Self>,
    outputter: WorkflowStep<'a, GeneratorOut, Self>,
  ) -> Self {
    Self {
      settings,
      tokenizer,
      asterizer,
      translator,
      checker,
      generator,
      outputter,
      jobs: vec![],
    }
  }

  pub(crate) fn compile(&'a mut self) -> Result<OutputterOut, String> {
    let entry_point = self.settings.input_file.to_owned();
    self.jobs.push(CompilerJob::Unprocessed(entry_point));

    let mut compiler = self;

    while let Some(job) = compiler.jobs.pop() {
      compiler = match job {
        CompilerJob::Unprocessed(path_buf) => (compiler.tokenizer)(compiler, path_buf)?,
        CompilerJob::Tokenized(tokenized) => (compiler.asterizer)(compiler, tokenized)?,
        CompilerJob::Asterized(asterized) => (compiler.translator)(compiler, asterized)?,
        CompilerJob::Translated(translated) => (compiler.checker)(compiler, translated)?,
        CompilerJob::Checked(checked) => (compiler.generator)(compiler, checked)?,
        CompilerJob::Generated(generated) => (compiler.outputter)(compiler, generated)?,
      };
    };

    unreachable!("ran out of compile jobs!");
  }
}
