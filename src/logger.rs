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

  builder.filter_level(log::LevelFilter::Off);
  builder.parse_default_env();

  builder.format(colog::formatter(Logger));
  builder.init();

  info!("Initialized logger");
}
