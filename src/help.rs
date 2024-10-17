pub(super) fn print_help_text() {
  let full_executable_path = std::env::current_exe().unwrap();
  let executable = full_executable_path.file_name().unwrap().to_string_lossy();

  eprintln!("\
    Usage: {executable} [OPTION]... [INPUT]\n\
    \n\
    Options:\n  \
      -h, --help                             Shows this help message\n  \
      -i, --input=<FILE>                     Sets the program's entry file\n  \
      -o, --output=<FILE>                    Sets the program's output file\n  \
    \n\
    Tooling:\n  \
      --llc=<FILE>                           Path to the llc executable\n  \
      --cc=<FILE>                            Path to the cc executable\n  \
    \n\
    See LICENSE for more information.\
  ")
}
