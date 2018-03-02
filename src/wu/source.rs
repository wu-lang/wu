use std::fs::File;
use std::io::prelude::*;

use std::fmt;

use colored::Colorize;

#[derive(Debug)]
pub struct FilePath(pub String);

impl fmt::Display for FilePath {
  fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "\n{:>8} {}", "-->".blue().bold(), self.0)
  }
}



#[derive(Debug)]
pub struct Source {
  pub file:  FilePath,
  pub lines: Vec<String>,
}

impl Source {
  pub fn new(path: String) -> Self {
    let mut source  = File::open(path.as_str()).unwrap();
    let mut content = String::new();

    source.read_to_string(&mut content).unwrap();

    Source {
      file:  FilePath(path),
      lines: content.lines().map(|x| x.to_string()).collect()
    }
  }

  pub fn from(path: &str, lines: Vec<String>) -> Self {
    Source {
      file: FilePath(path.into()),
      lines,
    }
  }
}