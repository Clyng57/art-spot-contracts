
use crate::*;

impl PartialEq<Id> for Id {
  fn eq(&self, other: &Self) -> bool {
      self.as_str() == other.as_str()
  }
}

impl PartialEq<str> for Id {
  fn eq(&self, other: &str) -> bool {
      if Id::is_near_id(other) {
          self.as_str() == other
      } else {
          self.as_str() == utils::unwrap(other.strip_suffix(Id::ARTSPOT_ID))
      }
  }
}

impl<'a> PartialEq<&'a str> for Id {
    fn eq(&self, other: &&'a str) -> bool {
      if Id::is_near_id(other) {
          self.as_str() == *other
      } else {
          self.as_str() == utils::unwrap(other.strip_suffix(Id::ARTSPOT_ID))
      }
    }
}

impl PartialEq<String> for Id {
  fn eq(&self, other: &String) -> bool {
    if Id::is_near_id(other) {
        self.as_str() == other.as_str()
    } else {
      self.as_str() == other.strip_suffix(Id::ARTSPOT_ID).unwrap()
    }
  }
}

impl PartialEq<AccountId> for Id {
  fn eq(&self, other: &AccountId) -> bool {
      self.to_account_id() == *other
  }
}

impl<'a> PartialEq<Id> for &'a str {
  fn eq(&self, other: &Id) -> bool {
    other == self
  }
}

impl PartialEq<Id> for String {
  fn eq(&self, other: &Id) -> bool {
    other == self
  }
}

impl PartialEq<Id> for AccountId {
  fn eq(&self, other: &Id) -> bool {
    other == self
  }
}

impl Eq for Id {}
