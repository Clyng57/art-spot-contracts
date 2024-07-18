
use std::{borrow::Borrow, fmt};

use near_sdk::{borsh::{BorshDeserialize, BorshSerialize}, serde::{Deserialize, Deserializer, Serialize, Serializer}, AccountId};

use crate::utils;

#[macro_export]
macro_rules! user_id {
    ($($arg:tt)*) => {{
        let res = ::std::fmt::format(::std::format_args!($($arg)*));
        Id::new(res)
    }}
}

pub fn is_valid(account_id: &str) -> bool {
  if account_id.len() < 2 {
      return false;
  } else if account_id.len() > 64 {
      return false;
  } else {
      // Adapted from https://github.com/near/near-sdk-rs/blob/fd7d4f82d0dfd15f824a1cf110e552e940ea9073/near-sdk/src/environment/env.rs#L819

      // NOTE: We don't want to use Regex here, because it requires extra time to compile it.
      // The valid account ID regex is /^(([a-z\d]+[-_])*[a-z\d]+\.)*([a-z\d]+[-_])*[a-z\d]+$/
      // Instead the implementation is based on the previous character checks.

      // We can safely assume that last char was a separator.
      let mut last_char_is_separator = true;

      for c in account_id.chars() {
          let current_char_is_separator = match c {
              'a'..='z' | '0'..='9' => false,
              '-' | '_' => true,
              _ => {
                  return false;
              }
          };

          if current_char_is_separator && last_char_is_separator {
              return false;
          }

          last_char_is_separator = current_char_is_separator;
      }

      if last_char_is_separator {
          return false;
      }

      true
  }
}

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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IdType {
  Near,
  ArtSpot,
}

#[derive(Clone, Debug, Hash)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "abi", derive(borsh::BorshSchema))]
pub struct Id(pub(crate) Box<str>, pub(crate) IdType);

impl Id {
  pub const MIN_LENGTH: usize = 2;
  pub const MAX_LENGTH: usize = 64;
  pub const ARTSPOT_ID: &'static str = "art-spot.near";
  pub const NEAR_ID: &'static str = "near";

  pub fn is_art_spot_id(username: &str) -> bool {
    username.ends_with(Self::ARTSPOT_ID)
  }

  pub fn is_near_id(username: &str) -> bool {
    !username.ends_with(Self::ARTSPOT_ID) && username.ends_with(Self::NEAR_ID)
  }

  pub fn validate(username: &str) -> Result<(), IdError> {
      if username.len() < Self::MIN_LENGTH {
          return Err(IdError::TooShort(username.len()));
      } else if username.len() > Self::MAX_LENGTH {
          return Err(IdError::TooLong(username.len()));
      }

      let mut last_char_is_separator = true;

      for c in username.chars() {
          let current_char_is_separator = match c {
              'a'..='z' | '0'..='9' => false,
              '-' | '_' => true,
              _ => {
                  return Err(IdError::Invalid(c));
              }
          };

          if current_char_is_separator && last_char_is_separator {
              return Err(IdError::Invalid(c));
          }

          last_char_is_separator = current_char_is_separator;
      }

      if last_char_is_separator {
          return Err(IdError::Invalid('-'));
      }

      Ok(())
  }

  pub fn is_valid(id: &str) -> bool {
      Self::validate(id).is_ok()
  }

  pub fn new(id: impl Into<String>) -> Self {
      let id = id.into();

      let (id, kind) = if id.ends_with(Self::ARTSPOT_ID) {
        (as_sdk::unwrap!(id.strip_suffix(Self::ARTSPOT_ID)), IdType::ArtSpot)
      } else if id.ends_with(Self::NEAR_ID) {
        (id.borrow(), IdType::Near)
      } else if id.len() == Self::MAX_LENGTH {
        (id.borrow(), IdType::Near)
      } else {
        (id.borrow(), IdType::ArtSpot)
      };

      if let Err(e) = Self::validate(&id) {
          crate::panic!("{}", e);
      }

      Self(Box::from(id), kind)
  }

  pub fn kind(&self) -> IdType {
      self.1.clone()
  }

  pub fn name(&self) -> Option<&str> {
    match self.1 {
        IdType::Near => {
  
            if self.0.len() == Id::MAX_LENGTH {
              return None
            }
    
            self.0.strip_suffix(Self::NEAR_ID)
        },
        IdType::ArtSpot => {
            Some(&self.0)
        }
    }
  }

  pub fn to_account_id(&self) -> AccountId {
      match self.1 {
          IdType::Near => as_sdk::parse_as!(self.0, AccountId),
          IdType::ArtSpot => as_sdk::parse_as!(format!("{}.{}", self.0, Self::ARTSPOT_ID), AccountId),
      }
  }

  pub fn to_string(&self) -> String {
      self.0.clone().into()
  }

  pub fn as_str(&self) -> &str {
      &self.0
  }

  pub fn as_bytes(&self) -> &[u8] {
      self.0.as_bytes()
  }
}

impl fmt::Display for Id {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      write!(f, "{}", self.0)
  }
}

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

impl Serialize for Id {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
      S: Serializer,
  {
      <str as Serialize>::serialize(&self.as_str(), serializer)
  }
}

impl<'de> Deserialize<'de> for Id {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
      D: Deserializer<'de>,
  {
      let id = <&str as Deserialize>::deserialize(deserializer)?;

      Ok(Id::new(id))
  }
}

impl BorshSerialize for Id {
  fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
    <str as BorshSerialize>::serialize(&self.as_str(), writer)
  }
}

impl BorshDeserialize for Id {
  fn deserialize(buf: &mut &[u8]) -> Result<Id, std::io::Error> {
    let id: String = <String as BorshDeserialize>::deserialize(buf).unwrap();
    Ok(Id::new(id))
  }

  fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
    let id: String = <String as BorshDeserialize>::deserialize_reader(reader).unwrap();
    Ok(Id::new(id))
  }
}

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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_id_new() {
    let id = Id::new("alice.near");
    assert_eq!(id.0, "alice.near".into());
    assert_eq!(id.1, IdType::Near);

    let id = Id::new("alice.art-spot.near");
    assert_eq!(id.0, "alice".into());
    assert_eq!(id.1, IdType::ArtSpot);

    let id = Id::new("alice");
    assert_eq!(id.0, "alice".into());
    assert_eq!(id.1, IdType::ArtSpot);

    let id = Id::new("alice.near");
    assert_eq!(id.0, "alice".into());
    assert_eq!(id.1, IdType::Near);
  }

  #[test]
  fn test_id_validate() {
    assert!(Id::validate("alice.near").is_ok());
    assert!(Id::validate("alice.art-spot.near").is_ok());
    assert!(Id::validate("alice").is_ok());
    assert!(Id::validate("alice.near").is_ok());

    assert!(Id::validate("alice..near").is_err());
    assert!(Id::validate("alice.-near").is_err());
    assert!(Id::validate("alice._near").is_err());
    assert!(Id::validate("alice.near.").is_err());
    assert!(Id::validate("alice.near-").is_err());
    assert!(Id::validate("alice.near_").is_err());
    assert!(Id::validate("alice.near..").is_err());
    assert!(Id::validate("alice.near--").is_err());
    assert!(Id::validate("alice.near__").is_err());
    assert!(Id::validate("alice.near.art-spot").is_err());
    assert!(Id::validate("alice.near.art-spot.near").is_err());
    assert!(Id::validate("a").is_err());
    assert!(Id::validate("a.near").is_err());
    assert!(Id::validate("a.art-spot.near").is_err());
    assert!(Id::validate("a.near.art-spot").is_err());
  }

  #[test]
  fn test_id_is_valid() {
    assert!(Id::is_valid("alice.near"));
    assert!(Id::is_valid("alice.art-spot.near"));
    assert!(Id::is_valid("alice"));
    assert!(Id::is_valid("alice.near"));

    assert!(!Id::is_valid("alice..near"));
    assert!(!Id::is_valid("alice.-near"));
    assert!(!Id::is_valid("alice._near"));
    assert!(!Id::is_valid("alice.near."));
    assert!(!Id::is_valid("alice.near-"));
    assert!(!Id::is_valid("alice.near_"));
    assert!(!Id::is_valid("alice.near.."));
    assert!(!Id::is_valid("alice.near--"));
  }

  #[test]
  fn test_id_name() {
    let id = Id::new("alice.near");
    assert_eq!(id.name(), Some("alice"));

    let id = Id::new("alice.art-spot.near");
    assert_eq!(id.name(), Some("alice.art-spot"));

    let id = Id::new("alice");
    assert_eq!(id.name(), Some("alice"));

    let id = Id::new("alice.near");
    assert_eq!(id.name(), None);
  }
}
