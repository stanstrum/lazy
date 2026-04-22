// SPONGE
pub struct CliArgs {
  pub target: Option<String>,
  pub opt_level: OptimizationLevel,
  pub passes: String,
}

#[derive(Clone, Copy)]
pub enum OptimizationLevel {
  O0,
  O1,
  O2,
  O3,
}

impl From<OptimizationLevel> for inkwell::OptimizationLevel {
  fn from(value: OptimizationLevel) -> Self {
    match value {
      OptimizationLevel::O0 => Self::None,
      OptimizationLevel::O1 => Self::Less,
      OptimizationLevel::O2 => Self::Default,
      OptimizationLevel::O3 => Self::Aggressive,
    }
  }
}
