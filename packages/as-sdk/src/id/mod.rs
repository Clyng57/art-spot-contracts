
mod error;
mod from;
mod serializers;
mod ord;
mod eq;

pub use error::IdError;
use near_sdk::NearSchema;
use schemars::JsonSchema;

#[macro_export]
macro_rules! user_id {
    ($($arg:tt)*) => {{
        let res = ::std::fmt::format(::std::format_args!($($arg)*));
        Id::new(res)
    }}
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum IdType {
  ArtSpot,
  Near,
  NearImplicit,
  Ethereum,
  Other,
}

impl IdType {
  pub fn as_str(&self) -> &str {
      match self {
          Self::Near => "near",
          Self::NearImplicit => "near-implicit",
          Self::ArtSpot => "art-spot",
          Self::Ethereum => "ethereum",
          Self::Other => "other",
      }
  }

  pub fn as_bytes(&self) -> &[u8] {
      self.as_str().as_bytes()
  }
}

impl std::fmt::Display for IdType {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
      write!(f, "{}", self.as_str())
  }
}

#[derive(Clone, Debug, Hash, NearSchema)]
pub struct Id(pub(crate) Box<str>, pub(crate) IdType);

impl Id {
  pub const MIN_LENGTH: usize = 2;
  pub const MAX_LENGTH: usize = 64;
  pub const ARTSPOT_ID: &'static str = "art-spot.near";
  pub const NEAR_ID: &'static str = "near";

  pub fn validate(id: &str) -> Result<(), IdError> {
      if id.len() < Self::MIN_LENGTH {
          return Err(IdError::TooShort(id.len()));
      } else if id.len() > Self::MAX_LENGTH {
          return Err(IdError::TooLong(id.len()));
      }

      let mut last_char_is_separator = true;

      for c in id.chars() {
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

      if id.ends_with(Self::ARTSPOT_ID) {
        let id: &str = crate::unwrap!(id.strip_suffix(Self::ARTSPOT_ID));
        crate::panic_on_error!(Self::validate(id));
        Self(Box::from(id), IdType::ArtSpot)
      } else if id.ends_with(Self::NEAR_ID) || is_near_implicit(&id) {
        crate::panic_on_error!(Self::validate(&id));
        Self(Box::from(id), IdType::Near)
      } else if is_eth_implicit(&id) {
        Self(Box::from(id), IdType::Ethereum)
      } else if id.contains('.') {
        crate::panic_on_error!(Self::validate(&id));
        Self(Box::from(id), IdType::Other)
      } else {
        crate::panic_on_error!(Self::validate(&id));
        Self(Box::from(id), IdType::ArtSpot)
      }
  }

  pub fn kind(&self) -> IdType {
      self.1.clone()
  }

  pub fn name(&self) -> Option<&str> {
    match self.1 {
        IdType::Near => {
            self.0.strip_suffix(Self::NEAR_ID)
        },
        IdType::NearImplicit => {
          None
        },
        IdType::ArtSpot => {
            Some(&self.0)
        },
        IdType::Ethereum => {
            None
        },
        IdType::Other => {
          self.0.rsplit_once('.').map(|(name, _)| name)
        }
    }
  }

  pub fn to_account_id(&self) -> near_sdk::AccountId {
      match self.1 {
          IdType::Near => crate::parse_as!(self.0, near_sdk::AccountId),
          IdType::ArtSpot => crate::parse_as!(format!("{}.{}", self.0, Self::ARTSPOT_ID), near_sdk::AccountId),
          IdType::Ethereum => crate::parse_as!(self.0, near_sdk::AccountId),
          IdType::NearImplicit => crate::parse_as!(self.0, near_sdk::AccountId),
          IdType::Other => crate::parse_as!(self.0, near_sdk::AccountId),
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

impl std::fmt::Display for Id {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
      write!(f, "{}", self.0)
  }
}

impl JsonSchema for Id {
  fn schema_name() -> String {
      "Id".to_string()
  }

  fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
    String::json_schema(gen)
  }
}

pub fn is_eth_implicit(id: &str) -> bool {
  id.len() == 42
      && id.starts_with("0x")
      && id[2..].as_bytes().iter().all(|b| matches!(b, b'a'..=b'f' | b'0'..=b'9'))
}

pub fn is_near_implicit(id: &str) -> bool {
  id.len() == 64
      && id
          .as_bytes()
          .iter()
          .all(|b| matches!(b, b'a'..=b'f' | b'0'..=b'9'))
}

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
