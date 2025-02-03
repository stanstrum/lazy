#[derive(Debug)]
pub enum Keyword {
  Type,
  Interface,
  Class,
  Struct,
  Implements,
  Extends,
  Namespace,
  Import,
  Export,
  From,
  For,
  While,
  Do,
  Loop,
  Until,
  In,
}

impl TryFrom<&str> for Keyword {
  type Error = ();

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "type" => Ok(Self::Type),
      "interface" => Ok(Self::Interface),
      "class" => Ok(Self::Class),
      "struct" => Ok(Self::Struct),
      "implements" => Ok(Self::Implements),
      "extends" => Ok(Self::Extends),
      "namespace" => Ok(Self::Namespace),
      "import" => Ok(Self::Import),
      "export" => Ok(Self::Export),
      "from" => Ok(Self::From),
      "for" => Ok(Self::For),
      "while" => Ok(Self::While),
      "do" => Ok(Self::Do),
      "loop" => Ok(Self::Loop),
      "until" => Ok(Self::Until),
      "in" => Ok(Self::In),
      _ => Err(()),
    }
  }
}
