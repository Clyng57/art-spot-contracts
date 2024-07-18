
use near_contract_standards::storage_management::{
  StorageBalance, StorageBalanceBounds, StorageManagement,
};

use near_sdk::{assert_one_yocto, log, StorageUsage};

use crate::*;
pub const MIN_STORAGE_BYTES: StorageUsage = 2000;

impl ArtSpot {
  pub(crate) fn internal_create_account(
      &mut self,
      id: &Id,
      storage_deposit: NearToken,
      registration_only: bool,
  ) {
      let min_balance = self.storage_balance_bounds().min;

      if storage_deposit.as_yoctonear() < min_balance.as_yoctonear() {
          env::panic_str("The attached deposit is less than the minimum storage balance");
      }

      let mut account = User::new(id.clone(), "".to_string(), utils::unwrap(id.name()).to_string());

      if registration_only {
          let refund = storage_deposit.saturating_sub(min_balance);

          if refund.as_yoctonear() > 0 {
              Promise::new(env::predecessor_account_id()).transfer(refund);
          }

          account.storage_balance = min_balance;
      } else {
          account.storage_balance = storage_deposit;
      }

      account.storage_tracker.start();

      require!(
          !self.users.insert(id.clone(), account.clone()).is_some(),
          "Internal bug. Account already exists."
      );

      account.storage_tracker.stop();
      self.internal_set_account(id, account);
  }

  pub(crate) fn internal_set_account(&mut self, id: &Id, mut account: User) -> bool {
      let bytes_added = account.storage_tracker.bytes_added;
      let bytes_released = account.storage_tracker.bytes_released;

      if bytes_added > bytes_released {
          let extra_bytes_used = bytes_added - bytes_released;
          account.used_bytes += extra_bytes_used;
          account.assert_storage_covered();
      } else if bytes_added < bytes_released {
          let bytes_released = bytes_released - bytes_added;

          require!(
              account.used_bytes >= bytes_released,
              "Internal storage accounting bug"
          );

          account.used_bytes -= bytes_released;
      }

      account.storage_tracker.bytes_released = 0;
      account.storage_tracker.bytes_added = 0;
      self.users.insert(id.clone(), account.clone()).is_some()
  }

  pub fn internal_storage_balance_of(&self, id: &Id) -> Option<StorageBalance> {
      self.internal_get_account(id)
          .map(|account| StorageBalance {
              total: account.storage_balance,
              available: account.storage_balance
                  .saturating_sub(
                    env::storage_byte_cost().saturating_mul(account.used_bytes.into())
                  ),
          })
  }

  /// Withdraw storage deposit from the account id to the predecessor account.
  /// Assumes that predecessor is authorized to withdraw from the account.
  pub fn internal_storage_withdraw(
      &mut self,
      withdraw_from: &Id,
      amount: Option<NearToken>,
  ) -> StorageBalance {
      if let Some(storage_balance) = self.internal_storage_balance_of(&withdraw_from) {
          let amount = amount.unwrap_or(storage_balance.available);

          if amount.as_yoctonear() > storage_balance.available.as_yoctonear() {
              env::panic_str("The amount is greater than the available storage balance");
          }

          if amount.as_yoctonear() > 0 {
              let mut account = self.internal_unwrap_account(withdraw_from);
              account.storage_balance = account.storage_balance.saturating_sub(amount);
              self.internal_set_account(withdraw_from, account);
              Promise::new(env::predecessor_account_id()).transfer(amount);
          }

          self.internal_storage_balance_of(&withdraw_from).unwrap()
      } else {
          env::panic_str(&format!("The account {} is not registered", &withdraw_from));
      }
  }

  pub fn internal_deposit_storage(&mut self, id: &Id, amount: NearToken) {
      let mut account = self.internal_unwrap_account(id);
      account.storage_balance = account.storage_balance.saturating_add(amount);
      self.internal_set_account(id, account);
  }
}

#[near]
impl StorageManagement for ArtSpot {
  // `registration_only` doesn't affect the implementation for vanilla fungible token.
  #[payable]
  #[allow(unused_variables)]
  fn storage_deposit(
      &mut self,
      account_id: Option<AccountId>,
      registration_only: Option<bool>,
  ) -> StorageBalance {
      let amount = env::attached_deposit();
      let id = Id::new(account_id.unwrap_or_else(env::predecessor_account_id));
      let registration_only = registration_only.unwrap_or(false);

      if registration_only {
        
      }
      if self.users.contains_key(&id) {
        if registration_only {
          log!("The account is already registered, refunding the deposit");
          if amount > NearToken::from_near(0) {
              Promise::new(env::predecessor_account_id()).transfer(amount);
          }
        } else {
          log!("The account is already registered, updating the storage balance");
          self.internal_deposit_storage(&id, amount);
        }
      } else {
          self.internal_create_account(&id, amount, registration_only);
      }

      self.internal_storage_balance_of(&id).unwrap()
  }

  /// While storage_withdraw normally allows the caller to retrieve `available` balance, the basic
  /// Fungible Token implementation sets storage_balance_bounds.min == storage_balance_bounds.max,
  /// which means available balance will always be 0. So this implementation:
  /// * panics if `amount > 0`
  /// * never transfers Ⓝ to caller
  /// * returns a `storage_balance` struct if `amount` is 0
  #[payable]
  fn storage_withdraw(&mut self, amount: Option<NearToken>) -> StorageBalance {
      assert_one_yocto();
      let predecessor_account_id = ctx::predecessor_id();

      if let Some(storage_balance) = self.internal_storage_balance_of(&predecessor_account_id) {
          match amount {
              Some(amount) if amount > NearToken::from_near(0) => {
                  env::panic_str("The amount is greater than the available storage balance");
              }
              _ => storage_balance,
          }
      } else {
          env::panic_str(
              format!("The account {} is not registered", &predecessor_account_id).as_str(),
          );
      }
  }

  #[allow(unused_variables)]
  #[payable]
  fn storage_unregister(&mut self, force: Option<bool>) -> bool {
    env::panic_str("The account can't be unregistered");
  }

  fn storage_balance_bounds(&self) -> StorageBalanceBounds {
    let required_storage_balance = env::storage_byte_cost().saturating_mul(
      MIN_STORAGE_BYTES.into()
    );

    StorageBalanceBounds {
      min: required_storage_balance,
      max: None
    }
  }

  fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
      self.internal_storage_balance_of(&Id::new(account_id))
  }
}
