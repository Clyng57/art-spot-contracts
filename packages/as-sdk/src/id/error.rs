use std::fmt;

pub enum IdError {
  Invalid(char),
  TooShort(usize),
  TooLong(usize),
}

impl fmt::Display for IdError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      match self {
          Self::Invalid(c) => write!(f, "Invalid character: '{}'", c),
          Self::TooShort(len) => write!(f, "Id is too short: {}", len),
          Self::TooLong(len) => write!(f, "Id is too long: {}", len),
      }
  }
}
