use colored::Colorize;

struct Logger;

impl colog::format::CologStyle for Logger {
  fn level_token(&self, level: &log::Level) -> &str {
    match *level {
      log::Level::Error => "error",
      log::Level::Warn => " warn",
      log::Level::Info => " info",
      log::Level::Debug => "debug",
      log::Level::Trace => "trace",
    }
  }

  fn level_color(&self, level: &log::Level, msg: &str) -> String {
    match *level {
      log::Level::Error => msg.red(),
      log::Level::Warn => msg.yellow(),
      log::Level::Info => msg.green(),
      log::Level::Debug => msg.bright_black(),
      log::Level::Trace => msg.magenta(),
    }.bold().to_string()
  }

  fn prefix_token(&self, level: &log::Level) -> String {
    format!(
      "{} {}",
      self.level_color(level, self.level_token(level)),
      "|".white().bold(),
    )
  }

  fn line_separator(&self) -> String {
    "\n      | ".white().bold().to_string()
  }
}

pub(super) fn init() {
  let mut builder = colog::basic_builder();

  builder.filter_level(log::LevelFilter::Info);
  builder.parse_default_env();

  builder.format(colog::formatter(Logger));
  builder.init();

  debug!("initialized logger");
}

#[macro_export]
macro_rules! enchant {
  ($expr:expr) => {{
    use colored::Colorize;

    if cfg!(feature = "vscode_links") {
      let file = file!();
      let line = line!();
      let column = column!();

      let current_path = std::env::current_dir().unwrap();
      let source_path_string = current_path.join(file)
        .canonicalize()
        .expect("couldn't find source file, do you need vscode_links?")
        .to_string_lossy()
        .to_string();

      format!("\x1b]8;;vscode://file{source_path_string}:{line}:{column}\x1b\\{}\x1b]8;;\x1b\\", $expr)
    } else {
      $expr.into()
    }.bold()
  }};
}
