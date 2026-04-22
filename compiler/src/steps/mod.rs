mod debug;

use std::{os::unix::fs::PermissionsExt, path::Path, process::ExitCode};

use crate::{Lazy, error, lang::reference::ModuleReference, print_message};

impl<'pool> Lazy<'pool> {
  pub fn check(&mut self) -> Result<ModuleReference, error::PrintableMessage> {
    // Instantiate the global scope
    let path = self.settings.input_path.to_owned();
    let global = self.add_file("@global", path, None)?;

    // Resolve, verify
    crate::resolve::resolve_and_verify(self, global)?;

    // Debugs
    debug::source(self, &global);
    debug::string_pool(self);

    Ok(global)
  }

  pub fn build(&mut self) -> Result<&Path, error::PrintableMessage> {
    let global = self.check()?;

    // Otherwise, let's go build the module
    let args = crate::generate::args::CliArgs {
      target: None,
      opt_level: crate::generate::args::OptimizationLevel::O0,
      passes: "instcombine,reassociate,gvn,simplifycfg,mem2reg".into(),
    };

    let program = crate::generate::Program::new(global, args);
    let compilation = program.compile(self)?;

    // Debug the LLVM source
    debug::llvm_source(self, &compilation);

    // Optimize the IR
    compilation.optimize(self)?;
    debug::llvm_source(self, &compilation);

    // Write out the object file for the global module
    // TODO: get this from settings
    let file_type = inkwell::targets::FileType::Object;
    let object_file = compilation.save_to_file(file_type)?;

    debug::object_file(self, &object_file);

    // TODO: find out what needs to be linked
    let linked = [
      "c", // libc
    ];

    // Link the module and write the program to the output file
    let executable = object_file.link_with(&self.settings.output_path, &linked)?;

    // Set the output file's permissions to be executable
    let perms = std::fs::Permissions::from_mode(0o755);
    std::fs::set_permissions(executable, perms)
      .expect("failed to chmod 755 {executable:?}");

    Ok(executable)
  }

  pub fn run(&mut self) -> Result<ExitCode, error::PrintableMessage> {
    let executable = self.build()?;

    // Otherwise, go run the child program
    let mut command = std::process::Command::new(executable);
    debug::subprocess_command(self, &command);

    let mut child = command
      // .args(args);
      .spawn()
      .expect("to launch {executable:?}");

    // Wait on the child and get an exit status
    let exit_status = child.wait()
      .expect("to wait on child process");

    // Print that info and set our own exit code accordingly
    let (exit_code, level, message) = if exit_status.success() {
      (
        ExitCode::SUCCESS, error::Level::Info,
        "Program exited successfully.".into(),
      )
    } else if let Some(code) = exit_status.code() {
      (
        ExitCode::FAILURE, error::Level::Error,
        format!("Program exited with status code {code}."),
      )
    } else {
      (
        ExitCode::FAILURE, error::Level::Error,
        "Program exited unsuccessfully.".into(),
      )
    };

    print_message!(self, {
      level,
      force: false,
      description: message,
      contents: MessageContents::None,
    });

    Ok(exit_code)
  }
}
