
extern crate as_macro;

mod yocto_near;
mod id;
mod utils;
mod error;
mod types;
mod version;

pub mod ctx;

use std::collections::HashMap;

pub use id::Id;
pub use yocto_near::YoctoNear;
pub use types::*;
pub use as_macro::{Ownable, Allowlist};
pub use version::Version;

/// Panic with a message.
#[macro_export]
macro_rules! panic {
    ($($arg:tt)*) => {{
        let res = ::std::fmt::format(::std::format_args!($($arg)*));
        near_sdk::env::panic_str(&res);
    }}
}

/// Require a condition to be true, otherwise panic with a message.
#[macro_export]
macro_rules! require {
    ($cond:expr $(,)?) => {
        if !$cond {
          near_sdk::env::panic_str("require! assertion failed");
        }
    };
    ($cond:expr, $($arg:tt)*) => {
        if !$cond {
            let message = ::std::fmt::format(::std::format_args!($($arg)*));
            near_sdk::env::panic_str(&message)
        }
    };
}

/// Unwrap an option, otherwise panic with a message.
#[macro_export]
macro_rules! unwrap {
    ($option:expr $(,)?) => {
      match $option {
        Some(val) => val,
        None => near_sdk::env::panic_str("unwrap! called on a None value"),
      }
    };
    ($option:expr, $($arg:tt)*) => {
        match $option {
            Some(val) => val,
            None => {
                let message = ::std::fmt::format(::std::format_args!($($arg)*));
                near_sdk::env::panic_str(&message)
            }
        }
    };
}

/// Parse a value, otherwise panic with a message.
#[macro_export]
macro_rules! parse_as {
    ($val:expr, $type:ty $(,)?) => {
        match $val.parse::<$type>() {
            Ok(val) => val,
            Err(_) => near_sdk::env::panic_str("Failed to parse value"),
        }
    };
    ($val:expr, $type:ty, $($arg:tt)*) => {
        match $val.parse::<$type>() {
            Ok(val) => val,
            Err(_) => {
                let message = ::std::fmt::format(::std::format_args!($($arg)*));
                near_sdk::env::panic_str(&message)
            }
        }
    };
}

/// Panic with an error message if the result is an error.
#[macro_export]
macro_rules! panic_on_error {
    ($result:expr $(,)?) => {
        match $result {
            Ok(val) => val,
            Err(e) => crate::panic!("{}", e),
        }
    };
}

#[macro_export]
macro_rules! hashmap {
    (// $keytype:expr, $valuetype:expr
      $($key:expr => $value:expr),* $(,)?) => {
        {
            let mut map = std::collections::HashMap::new();
            $(
                map.insert($key, $value);
            )*
            map
        }
    };
}

pub fn dead() -> HashMap<String, u8> {
  hashmap! {// String, u8
    "dead".to_string() => 0,
    String::from("second") => 1,
  }
}

pub trait Ownable {
  fn get_owner(&self) -> Id;
  fn set_owner(&mut self, owner: Id);
  fn is_owner(&self) -> bool {
    Id::from(near_sdk::env::predecessor_account_id()) == self.get_owner()
  }

  fn assert_owner(&self) {
    crate::require!(
      Id::from(near_sdk::env::predecessor_account_id()) == self.get_owner(),
      "Only owner can call this method"
    );
  }
}

pub trait Upgradable {
  fn staging_duration(&self) -> near_sdk::Duration;
  fn stage(&mut self, code: Vec<u8>, timestamp: near_sdk::Timestamp);
  fn deploy(&mut self) -> near_sdk::Promise;

  /// Implement migration for the next version.
  /// Should be `unimplemented` for a new contract.
  /// TODO: consider adding version of the contract stored in the storage?
  fn migrate(&mut self) {
      unimplemented!();
  }
}
