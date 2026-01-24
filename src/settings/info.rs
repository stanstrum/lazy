pub(super) fn version() {
  const LICENSE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/LICENSE"));
  let copyright = LICENSE.split('\n').next().unwrap();

  eprintln!("lazy {} - the laziest compiler ... zzz", env!("CARGO_PKG_VERSION"));
  eprintln!();
  eprintln!("{copyright}");
  eprintln!("Available under BSD 3-Clause \"New\" or \"Revised\" license.");
}

pub(super) fn help(executable: &str) {
  eprintln!("\x1b[92mUsage: \x1b[94m{executable} [VERB] INPUT [OPTIONS]...\x1b[0m");
  eprintln!();
  eprintln!("\x1b[92mVerbs:\x1b[0m");
  eprintln!("  \x1b[94mcheck, ck, c\x1b[0m            checks project for errors without compiling");
  eprintln!("  \x1b[94mbuild, b\x1b[0m                builds project");
  eprintln!("  \x1b[94mrun, r\x1b[0m                  builds and runs project");
  eprintln!();
  eprintln!("\x1b[92mOptions:\x1b[0m");
  eprintln!("  \x1b[94m--output-file=<FILE>\x1b[0m    sets the output file for the executable,");
  eprintln!("  \x1b[94m--output, -o=<FILE>\x1b[0m     defaults to \"./a.out\"");
  eprintln!("  \x1b[94m--log-level=<LEVEL>\x1b[0m     sets the log level, options are one of:");
  eprintln!("  \x1b[94m--log, -l=<LEVEL>\x1b[0m       debug, error, warn, info");
  eprintln!();
}
