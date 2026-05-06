use std::cmp::Ordering;

use lang::{CompilerPoolStore, reference::Store};

use super::*;

pub fn print_message<C: Compiler>(store: &C::Store<'_>, message: PrintableMessage<C>) {
  // TODO: Add settings (incl. log level) to `Lazy`

  // SPONGE: move this code into the macro
  // // If this message isn't being force-printed, check if we
  // // should actually print it
  // if !message.force && message.level < store.settings.log_level {
  //   // Don't print.
  //   return;
  // };

  // Output buffer for building the message
  let mut out = vec![];

  // "info: this is a message"
  writeln!(&mut out, "{bold}{level}{clear} {desc}",
    bold = colorize!(1),
    level = message.level,
    clear = colorize!(0),
    desc = message.description,
  ).unwrap();

  match message.contents {
    MessageContents::WithinSource(within_source) => {
      for WithinSource { range, sections } in within_source {
        print_sections(&mut out, store, range, sections);
      };
    },
    MessageContents::File(module) => {
      print_partial_section_header::<C>(&mut out, store, module);
    },
    MessageContents::None => {},
  };

  let out = std::str::from_utf8(&out).expect("output parsed as utf-8");
  print!("{out}");
}

fn print_section_header<C: Compiler>(out: &mut Vec<u8>, store: &C::Store<'_>, module: C::ModuleReference, position: Option<Position>) {
  let mut path = store.get_path(module).path.as_path();

  // SPONGE: reimplement this elsewhere
  // if
  //   let Some(parent) = store.settings.input_path.parent() &&
  //   let Ok(stripped) = path.strip_prefix(parent)
  // {
  //   path = stripped;
  // };

  write!(out, "  --> {}", path.to_string_lossy()).unwrap();

  if let Some(position) = position {
    write!(out, ":{line}:{col}",
      line = position.line,
      col = position.column,
    ).unwrap();
  };

  writeln!(out).unwrap();
}

fn print_partial_section_header<C: Compiler>(out: &mut Vec<u8>, store: &C::Store<'_>, module: C::ModuleReference) {
  print_section_header::<C>(out, store, module, None)
}

fn print_full_section_header<C: Compiler>(out: &mut Vec<u8>, store: &C::Store<'_>, span: lang::span::Span<C>) {
  print_section_header::<C>(out, store, span.module, Some(span.start))
}

fn print_sections<C: Compiler>(out: &mut Vec<u8>, store: &C::Store<'_>, range: lang::span::Span<C>, mut sections: Vec<MessageSection<C>>) {
  // Open and create a reader for this module's source file
  let lang::module::ModulePath::<C> { path, tokens, .. } = store.get_path(range.module);
  let file = File::open(path).unwrap();
  let mut reader = BufReader::new(file);

  // Get reference to the tokens saved by the asterizer
  let tokens = store.rget(*tokens).as_slice();
  let mut colorizer = Colorizer { tokens };

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

    if yielder.line > section.span.start.line {
      yielder.rewind(section.span.start);
    };

    print_full_section_header(out, store, section.span);

    // Skip at least until the line before
    while yielder.line + 1 < section.span.start.line {
      // skip line. TODO: this allocates a buffer and should be replaced w/ a
      // method that just drops these values
      yielder.next();
    };

    // Print the padding line(s) until we arrive
    while let line = yielder.line && line < section.span.start.line &&
      let Some(line_text) = yielder.next()
    {
      write!(out, " {line:>number_length$} {bold}|{clear} ",
        bold = colorize!(1),
        clear = colorize!(0),
      ).unwrap();
      colorizer.write(&mut *out, yielder.line, &line_text);
      writeln!(out).unwrap();
    };

    // Sanity check, that we are where we think we are
    assert!(yielder.line == section.span.start.line);

    // Print the lines in question, plus squiggles
    let lines_to_print = section.span.end.line - section.span.start.line;
    for _ in 0..=lines_to_print {
      let line = yielder.line;
      let line_text = yielder.next().expect("yield line");
      write!(out, " {bold}{line:>number_length$} |{clear} ",
        bold = colorize!(1),
        clear = colorize!(0),
      ).unwrap();
      colorizer.write(&mut *out, line, &line_text);
      writeln!(out).unwrap();

      let line_length = line_text.len();
      let squiggle_start = match line.cmp(&section.span.start.line) {
        Ordering::Less => panic!("out of bounds"),
        Ordering::Equal => section.span.start.column,
        Ordering::Greater => yielder.indentation + 1,
      };

      let squiggle_end = match line.cmp(&section.span.end.line) {
        Ordering::Less => {
          if line_length != 0 {
            line_length + 1
          } else {
            0
          }
        },
        Ordering::Equal => section.span.end.column,
        Ordering::Greater => panic!("out of bounds"),
      };

      let mut squiggle_text = (1..squiggle_end).map(|column| {
        if (squiggle_start..squiggle_end).contains(&column) {
          '^'
        } else {
          ' '
        }
      }).collect::<String>();

      if squiggle_start == squiggle_end {
        squiggle_text.push('^');
      };

      writeln!(out, " {number_padding} {bold}|{clear} {squiggle_text} {msg}",
        bold = colorize!(1),
        clear = colorize!(0),
        msg = if line == section.span.end.line { &section.text } else { "" },
      ).unwrap();
    };
  };
}
