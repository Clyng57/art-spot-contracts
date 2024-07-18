
use near_sdk::{Gas, Promise};
use as_sdk::Upgradable;
use crate::*;

const NO_ARGS: Vec<u8> = vec![];
const CALL_GAS: Gas = Gas::from_tgas(200); // 200 TGAS

#[near]
impl Upgradable for ArtSpot {
  pub fn staging_duration(&self) -> Duration {
      self.staging_duration
  }

  pub fn stage(&mut self, code: Vec<u8>, timestamp: Timestamp) {
      self.assert_owner();
      as_sdk::require!(
          env::block_timestamp() + self.staging_duration < timestamp,
          "Timestamp must be later than staging duration"
      );
      // Writes directly into storage to avoid serialization penalty by using default struct.
      env::storage_write(b"upgrade", &code);
      self.staging_timestamp = timestamp;
  }


  pub fn deploy(&mut self) -> Promise {
    self.assert_owner();

    if self.staging_timestamp < env::block_timestamp() {
        env::panic_str(
            format!(
                "Deploy code too early: staging ends on {}",
                self.staging_timestamp + self.staging_duration
            )
            .as_str(),
        );
    }

    let code: Vec<u8> = env::storage_read(b"upgrade").unwrap_or_else(|| env::panic_str("No upgrade code available"));
    env::storage_remove(b"upgrade");

    Promise::new(env::current_account_id()).deploy_contract(code)
        .function_call(
            "migrate".to_string(),
            NO_ARGS,
            NearToken::from_near(0),
            CALL_GAS,
        )
        .as_return()
  }
}

#[near]
impl ArtSpot {
  pub fn update(&self) -> Promise {
    self.assert_owner();
    // Receive the code directly from the input to avoid the
    // GAS overhead of deserializing parameters
    let code = env::input().expect("Error: No input").to_vec();

    // Deploy the contract on self
    Promise::new(env::current_account_id())
        .deploy_contract(code)
        .function_call(
            "migrate".to_string(),
            NO_ARGS,
            NearToken::from_near(0),
            CALL_GAS,
        )
        .as_return()
  }
}
