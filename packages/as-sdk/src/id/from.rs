
use near_sdk::AccountId;
use crate::*;

impl From<&str> for Id {
  fn from(id: &str) -> Self {
      Self::new(id)
  }
}

impl From<String> for Id {
  fn from(id: String) -> Self {
      Self::new(id)
  }
}

impl From<Id> for String {
  fn from(id: Id) -> String {
      id.to_string()
  }
}

impl From<Id> for AccountId {
  fn from(id: Id) -> AccountId {
      id.to_account_id()
  }
}

impl From<AccountId> for Id {
  fn from(id: AccountId) -> Id {
      Id::new(id.to_string())
  }
}
