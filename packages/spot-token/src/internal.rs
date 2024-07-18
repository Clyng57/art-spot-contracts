use near_sdk::require;

use crate::*;

const ERR_TOTAL_SUPPLY_OVERFLOW: &str = "Total supply overflow";

pub(crate) fn default_ft_metadata() -> FungibleTokenMetadata {
  FungibleTokenMetadata {
      spec: FT_METADATA_SPEC.to_string(),
      name: "ArtSpot Token".to_string(),
      symbol: "SPOT".to_string(),
      icon: Some(String::from(
          r#"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 80.03 80.04'%3E%3Cg%3E%3Cpath fill='currentColor' d='M74.88,27.45c-2.92-2.4-6.03-3.93-9.25-4.56l-.2-.04c-4.75-1.16-14.89-.18-18.7,3.51-1.56,1.51-1.63,3.06-1.42,4.08.22,1.11.93,2.22,2.31,3.62.74.73,1.53,1.46,2.37,2.23,2.43,2.23,4.95,4.53,6.25,7.19,2.01,3.9,2.12,8.74.29,12.94-1.73,3.99-4.99,6.92-8.96,8.05-8.38,2.62-16.62-1.73-20.5-10.82-2.33-5.7-2.43-12.15-.31-19.13,1.9-5.75,6.05-9.84,13.05-12.88,10.31-4.48,20.92-5.46,29.12-2.68.81.27,1.5.6,2.05.86.89.42,1.66.79,2.36.17.15-.13.63-.64.35-1.48l-.09-.21c-2.62-4.71-9.73-11.23-14.85-13.65-11.5-6.29-25.54-6.22-37.56.2C9.23,11.26,1.39,22.81.23,35.76c-.65,6,.08,12.14,2.09,17.73.84,2.39,1.85,4.7,3.4,5.33.64.3,1.33.33,1.96.07.76-.31,1.38-1,1.71-1.89.79-2.05.52-4.14.26-6.16-.19-1.43-.36-2.78-.13-4.1.16-1.64,1.38-3.19,3.04-3.84,1.41-.56,2.82-.35,3.89.59,1.7,1.44,2.02,4.62,2.1,7.25.04.82.07,1.64.11,2.47.29,7.45.6,15.16,6.42,21.11,4.1,3.79,9.41,5.72,15.55,5.72,1.84,0,3.76-.17,5.75-.52,18.66-2.88,32.8-18.67,33.61-37.58v-1c.1-5.12.17-9.54-5.12-13.49ZM22.49,26.96c-.17,3.92-3.22,5.95-6.16,5.95-.02,0-.04,0-.07,0-2.94-.03-5.97-2.13-6.06-6.07.08-3.99,3.11-6.09,6.06-6.12,2.98-.02,6.06,2,6.23,5.95v.27s0,.03,0,.03Z'/%3E%3C/g%3E%3C/svg%3E"#,
      )),
      reference: None,
      reference_hash: None,
      decimals: 18,
  }
}

impl SpotToken {
  pub(crate) fn assert_owner(&self) {
      assert!(
          env::predecessor_account_id() == self.owner_id,
          "can only be called by the owner"
      );
  }

  pub(crate) fn assert_minter(&self, account_id: &AccountId) {
      assert!(self.minters.contains(account_id), "not a minter");
  }

  pub(crate) fn mint_into(&mut self, account_id: &AccountId, amount: Balance) {
    let balance = self.internal_unwrap_balance_of(account_id);
    if let Some(new_balance) = balance.checked_add(amount) {
        self.accounts.insert(&account_id, &new_balance);
        self.total_supply = self
            .total_supply
            .checked_add(amount)
            .unwrap_or_else(|| env::panic_str(ERR_TOTAL_SUPPLY_OVERFLOW));
    } else {
        env::panic_str("Balance overflow");
    }
  }

  pub(crate) fn internal_burn(&mut self, account_id: &AccountId, amount: Balance) {
      let balance = self.internal_unwrap_balance_of(account_id);
      if let Some(new_balance) = balance.checked_sub(amount) {
          self.accounts.insert(&account_id, &new_balance);
          self.total_supply = self
              .total_supply
              .checked_sub(amount)
              .unwrap_or_else(|| env::panic_str(ERR_TOTAL_SUPPLY_OVERFLOW));
      } else {
          env::panic_str("The account doesn't have enough balance");
      }
  }

  //get stored metadata or default
  pub(crate) fn internal_get_ft_metadata(&self) -> FungibleTokenMetadata {
      self.metadata.get().unwrap_or(default_ft_metadata())
  }

  pub(crate) fn internal_unwrap_balance_of(&self, account_id: &AccountId) -> Balance {
      match self.accounts.get(account_id) {
          Some(balance) => balance,
          None => {
              env::panic_str(format!("The account {} is not registered", &account_id).as_str())
          }
      }
  }

  pub(crate) fn internal_deposit(&mut self, account_id: &AccountId, amount: Balance) {
      let balance = self.internal_unwrap_balance_of(account_id);
      if let Some(new_balance) = balance.checked_add(amount) {
          self.accounts.insert(&account_id, &new_balance);
          self.total_supply = self
              .total_supply
              .checked_add(amount)
              .unwrap_or_else(|| env::panic_str(ERR_TOTAL_SUPPLY_OVERFLOW));
      } else {
          env::panic_str("Balance overflow");
      }
  }

  pub(crate) fn internal_withdraw(&mut self, account_id: &AccountId, amount: Balance) {
      let balance = self.internal_unwrap_balance_of(account_id);
      if let Some(new_balance) = balance.checked_sub(amount) {
          self.accounts.insert(&account_id, &new_balance);
          self.total_supply = self
              .total_supply
              .checked_sub(amount)
              .unwrap_or_else(|| env::panic_str(ERR_TOTAL_SUPPLY_OVERFLOW));
      } else {
          env::panic_str("The account doesn't have enough balance");
      }
  }

  pub(crate) fn internal_transfer(
      &mut self,
      sender_id: &AccountId,
      receiver_id: &AccountId,
      amount: Balance,
      memo: Option<String>,
  ) {
      require!(sender_id != receiver_id, "Sender and receiver should be different");
      require!(amount > 0, "The amount should be a positive number");
      self.internal_withdraw(sender_id, amount);
      self.internal_deposit(receiver_id, amount);
      FtTransfer {
          old_owner_id: sender_id,
          new_owner_id: receiver_id,
          amount: U128(amount),
          memo: memo.as_deref(),
      }
      .emit();
  }

  pub(crate) fn internal_register_account(&mut self, account_id: &AccountId) {
      if self.accounts.insert(&account_id, &0).is_some() {
          env::panic_str("The account is already registered");
      }
  }

  /// Internal method that returns the amount of burned tokens in a corner case when the sender
  /// has deleted (unregistered) their account while the `ft_transfer_call` was still in flight.
  /// Returns (Used token amount, Burned token amount)
  pub(crate) fn internal_ft_resolve_transfer(
      &mut self,
      sender_id: &AccountId,
      receiver_id: AccountId,
      amount: U128,
  ) -> (u128, u128) {
      let amount: Balance = amount.into();

      // Get the unused amount from the `ft_on_transfer` call result.
      let unused_amount = match env::promise_result(0) {
          PromiseResult::Successful(value) => {
              if let Ok(unused_amount) = near_sdk::serde_json::from_slice::<U128>(&value) {
                  std::cmp::min(amount, unused_amount.0)
              } else {
                  amount
              }
          }
          PromiseResult::Failed => amount,
      };

      if unused_amount > 0 {
          let receiver_balance = self.accounts.get(&receiver_id).unwrap_or(0);
          if receiver_balance > 0 {
              let refund_amount = std::cmp::min(receiver_balance, unused_amount);
              if let Some(new_receiver_balance) = receiver_balance.checked_sub(refund_amount) {
                  self.accounts.insert(&receiver_id, &new_receiver_balance);
              } else {
                  env::panic_str("The receiver account doesn't have enough balance");
              }

              if let Some(sender_balance) = self.accounts.get(sender_id) {
                  if let Some(new_sender_balance) = sender_balance.checked_add(refund_amount) {
                      self.accounts.insert(&sender_id, &new_sender_balance);
                  } else {
                      env::panic_str("Sender balance overflow");
                  }

                  FtTransfer {
                      old_owner_id: &receiver_id,
                      new_owner_id: sender_id,
                      amount: U128(refund_amount),
                      memo: Some("refund"),
                  }
                  .emit();
                  let used_amount = amount
                      .checked_sub(refund_amount)
                      .unwrap_or_else(|| env::panic_str(ERR_TOTAL_SUPPLY_OVERFLOW));
                  return (used_amount, 0);
              } else {
                  // Sender's account was deleted, so we need to burn tokens.
                  self.total_supply = self
                      .total_supply
                      .checked_sub(refund_amount)
                      .unwrap_or_else(|| env::panic_str(ERR_TOTAL_SUPPLY_OVERFLOW));
                  log!("The account of the sender was deleted");
                  FtBurn {
                      owner_id: &receiver_id,
                      amount: U128(refund_amount),
                      memo: Some("refund"),
                  }
                  .emit();
                  return (amount, refund_amount);
              }
          }
      }
      (amount, 0)
  }
}
