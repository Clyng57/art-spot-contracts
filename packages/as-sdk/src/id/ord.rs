
use crate::*;

impl Ord for Id {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
      self.0.cmp(&other.0)
  }
}

impl PartialOrd for Id {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
      Some(self.cmp(other))
  }
}
