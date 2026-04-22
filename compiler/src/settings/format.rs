fn format_hex_number(num: u8) -> char {
  assert!(num <= 15);

  (if num < 10 {
    b'0' + num
  } else {
    b'a' + (num - 10)
  }) as char
}

pub fn format_argument(arg: &str) -> String {
  let has_problem_characters = arg.chars().any(|ch| {
    !ch.is_ascii() || matches!(ch, ' ' | '"' | '(' | ')' | '!' | '$' | '`' | ';' | '\'' )
                   || ch <= ' ' || ch > '~'
  });

  let mut out = String::new();

  if has_problem_characters {
    out.push('"');
  };

  for ch in arg.chars() {
    match ch {
      '"' | '!' | '$' | '`' | '\\' => {
        out.push('\\');
        out.push(ch);
      },
      _ if !ch.is_ascii() || !(' '..='~').contains(&ch) => {
        let mut buffer = [0; 4];
        ch.encode_utf8(&mut buffer);

        let bytes = buffer.into_iter()
          .take_while(|byte| *byte != 0);

        out.push_str("\"$'");
        for byte in bytes {
          out.push_str("\\x");
          out.push(format_hex_number(byte / 16));
          out.push(format_hex_number(byte % 16));
        };
        out.push_str("'\"");
      },
      _ => out.push(ch),
    }
  };

  if has_problem_characters {
    out.push('"');
  };

  if out.starts_with("\"\"") {
    out.replace_range(..2, "");
  };

  if out.ends_with("\"\"") {
    out.truncate(out.len() - 2);
  };

  out
}

pub(super) fn show_error_position(args: Vec<String>, position: usize) {
  let formatted = args.iter()
    .map(|x| format_argument(x));

  for (i, arg) in formatted.enumerate() {
    let (start, end) = if i == position {
      ("\x1b[31;1;4m", "\x1b[0m")
    } else {
      ("", "")
    };

    eprint!("{space}{start}{arg}{end}",
      space = if i != 0 { " " } else { "" },
    );
  };

  eprintln!();
}

