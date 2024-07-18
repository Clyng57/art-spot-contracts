
use near_sdk::StorageUsage;
use storage_tracker::StorageTracker;

use crate::{*, id::*};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[near(serializers = [borsh, json])]
pub enum Role {
  Admin,
  Contract,
  Moderator,
  Developer,
  Artist,
  Collector,
}

impl Role {
  pub fn to_string(&self) -> String {
    match self {
      Self::Admin => "Admin".to_string(),
      Self::Contract => "Contract".to_string(),
      Self::Moderator => "Moderator".to_string(),
      Self::Developer => "Developer".to_string(),
      Self::Artist => "Artist".to_string(),
      Self::Collector => "Collector".to_string(),
    }
  }
}

impl From<&str> for Role {
  fn from(role: &str) -> Self {
    match role {
      "Admin" => Self::Admin,
      "Moderator" => Self::Moderator,
      "Contract" => Self::Contract, // "Contract" is a special role for the contract itself
      "Developer" => Self::Developer,
      "Artist" => Self::Artist,
      "Collector" => Self::Collector,
      _ => crate::panic!("Invalid role"),
    }
  }
}

impl From<String> for Role {
  fn from(role: String) -> Self {
    Self::from(role.as_str())
  }
}

impl From<Role> for String {
  fn from(role: Role) -> Self {
    role.to_string()
  }
}

impl From<Role> for u8 {
  fn from(role: Role) -> Self {
    match role {
      Role::Admin => 0,
      Role::Moderator => 1,
      Role::Developer => 2,
      Role::Artist => 3,
      Role::Collector => 4,
      Role::Contract => 5,
    }
  }
}

impl From<u8> for Role {
  fn from(role: u8) -> Self {
    match role {
      0 => Role::Admin,
      1 => Role::Moderator,
      2 => Role::Developer,
      3 => Role::Artist,
      4 => Role::Collector,
      5 => Role::Contract,
      _ => crate::panic!("Invalid role"),
    }
  }
}

#[derive(Clone, Debug)]
#[near(serializers = [borsh, json])]
pub struct User {
  pub storage_balance: NearToken,
  pub used_bytes: StorageUsage,
  pub id: Id,
  pub email: String,
  pub name: String,
  pub bio: Option<String>,
  pub avatar: Option<String>,
  pub cover: Option<String>,
  pub created_at: u64,
  pub updated_at: u64,
  pub roles: Vec<Role>,
  pub verified: bool,
  #[serde(skip)]
  #[borsh(skip)]
  pub storage_tracker: StorageTracker,
}

impl User {
  pub fn new(id: Id, email: String, name: String) -> Self {
    Self {
      storage_balance: NearToken::from_yoctonear(0),
      used_bytes: 0,
      id,
      email,
      name,
      bio: None,
      avatar: None,
      cover: None,
      created_at: env::block_timestamp(),
      updated_at: env::block_timestamp(),
      roles: vec![Role::Artist],
      verified: false,
      storage_tracker: StorageTracker::default(),
    }
  }

  pub(crate) fn assert_storage_covered(&self) {
      let storage_balance_needed = env::storage_byte_cost().saturating_mul(Balance::from(self.used_bytes));

      require!(
          storage_balance_needed.as_yoctonear() <= self.storage_balance.as_yoctonear(),
          "Not enough storage balance"
      );
  }
}
