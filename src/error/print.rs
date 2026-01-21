use super::*;

struct LineYielder {
  reader: BufReader<File>,
  line: usize,
  finished: bool,
}

impl LineYielder {
  fn new(reader: BufReader<File>, line: usize) -> Self {
    Self {
      reader,
      line,
      finished: false,
    }
  }
}

impl Iterator for LineYielder {
  type Item = String;

  fn next(&mut self) -> Option<Self::Item> {
    if self.finished {
      return None;
    };

    let mut buffer = String::new();
    let remaining = self.reader.read_line(&mut buffer)
      .expect("read line");

    if remaining == 0 {
      self.finished = true;
    };

    self.line += 1;
    Some(buffer)
  }
}

pub fn print_message(lazy: &Lazy, message: PrintableMesage) {
  // TODO: Add settings (incl. log level) to `Lazy`
  //
  // // If this message isn't being force-printed, check if we
  // // should actually print it
  // if !message.force && message.level < lazy.settings.log_level {
  //   // Don't print.
  //   return;
  // };

  // Output buffer for building the message
  let mut out = vec![];

  // "info: this is a message"
  writeln!(&mut out, "{level}: {desc}",
    level = message.level.to_string().to_lowercase(),
    desc = message.description,
  ).unwrap();

  if let MessageContents::WithinSource { range, sections } = message.contents {
    print_sections(&mut out, lazy, range, sections);
  };

  let out = std::str::from_utf8(&out).expect("output parsed as utf-8");
  println!("{out}");
}

fn print_sections(out: &mut Vec<u8>, lazy: &Lazy, range: Span, mut sections: Vec<MessageSection>) {
  // Open and create a reader for this module's source file
  let path = lazy.get_path(range.module);
  let file = File::open(path).unwrap();
  let mut reader = BufReader::new(file);

  // We'll track where we are in the file once we start
  // moving around for coloring the code, printing the
  // line numbers reliably, and finally underlining relevant
  // parts of the code.

  // // We'll be making sure this is the case.
  // let mut column = 1;

  // The provided range might not land on the character
  // after a newline, i.e., in the middle of a line.  Find
  // the preceding newline, if there is one.
  let _end_position = seek::find_ending_newline(
    &mut reader,
    range.end.position,
  );

  let _start_position = seek::find_starting_newline(
    &mut reader,
    range.start.position
  );

  // Sort our sections so we can print them in progressive order
  sections.sort_by_key(|section| section.span.start.position);

  // String length of the greatest line number we find
  let number_length = (range.end.line.ilog10() + 1) as usize;
  let number_padding = " ".repeat(number_length);

  // Keep track of lines and yield one at a time
  let mut yielder = LineYielder::new(reader, range.start.line);

  // Print each section
  for section in sections.iter() {
    let this_line = section.span.start.line;

    writeln!(out, "  --> {path}:{line}:{col}",
      path = lazy.get_path(section.span.module).to_string_lossy(),
      line = this_line,
      col = section.span.start.column,
    ).unwrap();

    // Skip at least until the line before
    while yielder.line + 1 < section.span.start.line {
      // skip line. TODO: this allocates a buffer and should be replaced w/ a
      // method that just drops these values
      yielder.next();
    };

    // Print the padding line(s) until we arrive
    while yielder.line < section.span.start.line &&
      let Some(line_text) = yielder.next()
    {
      writeln!(out, " {line:>number_length$} | {line_text}",
        line = yielder.line,
      ).unwrap();
    };

    // Sanity check, that we are where we think we are
    assert!(yielder.line == section.span.start.line);

    // Print the lines in question, plus squiggles
    let lines_to_print = section.span.end.line - section.span.start.line;
    for _ in 0..=lines_to_print {
      let line = yielder.line;
      let line_text = yielder.next().expect("yield line");
      let line_text = line_text.trim_end_matches(['\r', '\n']);
      writeln!(out, " {line:>number_length$} | {line_text}").unwrap();

      let line_length = line_text.len();
      let squiggle_start = match line.cmp(&section.span.start.line) {
        std::cmp::Ordering::Less => panic!("out of bounds"),
        std::cmp::Ordering::Equal => section.span.start.column,
        std::cmp::Ordering::Greater => 1,
      };

      let squiggle_end = match line.cmp(&section.span.end.line) {
        std::cmp::Ordering::Less => line_length,
        std::cmp::Ordering::Equal => section.span.end.column,
        std::cmp::Ordering::Greater => panic!("out of bounds"),
      };

      let squiggle_text = (1..squiggle_end).map(|column| {
        if (squiggle_start..squiggle_end).contains(&column) {
          '^'
        } else {
          ' '
        }
      }).collect::<String>();

      writeln!(out, " {number_padding} | {squiggle_text} {msg}", msg = section.text).unwrap();
    };
  };
}
