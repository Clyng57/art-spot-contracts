
use near_contract_standards::fungible_token::receiver::ext_ft_receiver;
use near_contract_standards::fungible_token::resolver::ext_ft_resolver;
use near_sdk::borsh::BorshSerialize;
use near_sdk::json_types::U128;
// use near_sdk::store::{LookupMap};
use near_sdk::collections::{LookupMap, LazyOption};
use near_sdk::{
    assert_one_yocto, env, log, near, require, AccountId, BorshStorageKey, Gas, PanicOnDefault, PromiseOrValue, PromiseResult, StorageUsage
};
use near_contract_standards::fungible_token::{
    core::FungibleTokenCore,
    metadata::{FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC},
    resolver::FungibleTokenResolver,
};

pub mod events;
mod storage;
mod internal;

use crate::events::*;

const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas::from_tgas(5);
const GAS_FOR_FT_TRANSFER_CALL: Gas = Gas::from_tgas(30);
const ICON: &str = r#"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' style='background:%23f5f5f5' viewBox='0 0 80.03 80.04'%3E%3Cg%3E%3Cpath fill='%23000' d='M74.88,27.45c-2.92-2.4-6.03-3.93-9.25-4.56l-.2-.04c-4.75-1.16-14.89-.18-18.7,3.51-1.56,1.51-1.63,3.06-1.42,4.08.22,1.11.93,2.22,2.31,3.62.74.73,1.53,1.46,2.37,2.23,2.43,2.23,4.95,4.53,6.25,7.19,2.01,3.9,2.12,8.74.29,12.94-1.73,3.99-4.99,6.92-8.96,8.05-8.38,2.62-16.62-1.73-20.5-10.82-2.33-5.7-2.43-12.15-.31-19.13,1.9-5.75,6.05-9.84,13.05-12.88,10.31-4.48,20.92-5.46,29.12-2.68.81.27,1.5.6,2.05.86.89.42,1.66.79,2.36.17.15-.13.63-.64.35-1.48l-.09-.21c-2.62-4.71-9.73-11.23-14.85-13.65-11.5-6.29-25.54-6.22-37.56.2C9.23,11.26,1.39,22.81.23,35.76c-.65,6,.08,12.14,2.09,17.73.84,2.39,1.85,4.7,3.4,5.33.64.3,1.33.33,1.96.07.76-.31,1.38-1,1.71-1.89.79-2.05.52-4.14.26-6.16-.19-1.43-.36-2.78-.13-4.1.16-1.64,1.38-3.19,3.04-3.84,1.41-.56,2.82-.35,3.89.59,1.7,1.44,2.02,4.62,2.1,7.25.04.82.07,1.64.11,2.47.29,7.45.6,15.16,6.42,21.11,4.1,3.79,9.41,5.72,15.55,5.72,1.84,0,3.76-.17,5.75-.52,18.66-2.88,32.8-18.67,33.61-37.58v-1c.1-5.12.17-9.54-5.12-13.49ZM22.49,26.96c-.17,3.92-3.22,5.95-6.16,5.95-.02,0-.04,0-.07,0-2.94-.03-5.97-2.13-6.06-6.07.08-3.99,3.11-6.09,6.06-6.12,2.98-.02,6.06,2,6.23,5.95v.27s0,.03,0,.03Z'/%3E%3C/g%3E%3C/svg%3E"#;

pub type Balance = u128;

#[derive(PanicOnDefault)]
#[near(contract_state)]
pub struct SpotToken {
    /// Keep track of each account's balances
    pub accounts: LookupMap<AccountId, Balance>,

    /// Owner of the contract
    pub owner_id: AccountId,

    /// Minters are allowed to mint more tokens
    pub minters: Vec<AccountId>,

    /// Total supply of all tokens.
    pub total_supply: Balance,

    /// The storage size in bytes for one account.
    pub account_storage_usage: StorageUsage,

    /// Metadata for the contract itself
    metadata: LazyOption<FungibleTokenMetadata>,
}

#[near(serializers = [borsh])]
#[derive(BorshStorageKey)]
pub enum StorageKey {
    Accounts,
    Metadata
}

#[near]
impl SpotToken {
    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// default metadata (for example purposes only).
    #[init]
    #[private]
    pub fn new_default_meta(owner_id: AccountId, total_supply: U128) -> Self {
        // Calls the other function "new: with some default metadata and the owner_id & total supply passed in 
        Self::new(
            owner_id,
            total_supply,
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "ArtSpot Token".to_string(),
                symbol: "SPOT".to_string(),
                icon: Some(String::from(ICON)),
                reference: None,
                reference_hash: None,
                decimals: 18,
            },
        )
    }

    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// the given fungible token metadata.
    #[init]
    #[private]
    pub fn new(
        owner_id: AccountId,
        total_supply: U128,
        metadata: FungibleTokenMetadata,
    ) -> Self {
      require!(!env::state_exists(), "Already initialized");
      metadata.assert_valid();

        // Create a variable of type Self with all the fields initialized. 
        let mut this = Self {
            // Set the total supply
            total_supply: 0,
            owner_id: owner_id.to_owned(),
            minters: vec![owner_id.clone()],
            // Set the bytes for the longest account ID to 0 temporarily until it's calculated later
            account_storage_usage: 0,
            // Storage keys are simply the prefixes used for the collections. This helps avoid data collision
            accounts: LookupMap::new(StorageKey::Accounts),
            metadata: LazyOption::new(
                StorageKey::Metadata,
                Some(&metadata),
            ),
        };

        // Measure the bytes for the longest account ID and store it in the contract.
        this.measure_account_storage_usage();
        // Register the owner's account and set their balance to the total supply.
        this.internal_register_account(&owner_id);
        this.internal_deposit(&owner_id, total_supply.into());
        
        // Emit an event showing that the FTs were minted
        FtMint {
            owner_id: &owner_id,
            amount: total_supply,
            memo: Some("Initial token supply is minted"),
        }
        .emit();

        // Return the Contract object
        this
    }

    fn measure_account_storage_usage(&mut self) {
        let initial_storage_usage = env::storage_usage();
        let tmp_account_id: AccountId = "a".repeat(64).parse().unwrap();
        self.accounts.insert(&tmp_account_id, &0u128);
        self.account_storage_usage = env::storage_usage() - initial_storage_usage;
        self.accounts.remove(&tmp_account_id);
    }

    pub fn get_owner_id(&self) -> AccountId {
        return self.owner_id.clone();
    }

    #[payable]
    pub fn burn(&mut self, account_id: &AccountId, amount: U128, memo: Option<String>) {
        assert_one_yocto();
        self.assert_owner();
        let amount: Balance = amount.into();
        self.internal_burn(account_id, amount);

        FtBurn {
            owner_id: &account_id,
            amount: amount.into(),
            memo: memo.as_deref(),
        }.emit();
    }

    #[payable]
    pub fn mint(&mut self, account_id: &AccountId, amount: U128, memo: Option<String>) {
        assert_one_yocto();
        self.assert_minter(&env::predecessor_account_id());
        self.mint_into(account_id, amount.into());

        FtMint {
            owner_id: &account_id,
            amount,
            memo: memo.as_deref(),
        }.emit();
    }

    #[payable]
    pub fn add_minter(&mut self, account_id: AccountId) {
        assert_one_yocto();
        self.assert_owner();

        if let Some(_) = self.minters.iter().position(|x| *x == account_id) {
            //found
            panic!("already in the list");
        }
        self.minters.push(account_id);
    }

    #[payable]
    pub fn remove_minter(&mut self, account_id: &AccountId) {
        assert_one_yocto();
        self.assert_owner();

        if let Some(inx) = self.minters.iter().position(|x| x == account_id) {
            //found
            let _removed = self.minters.swap_remove(inx);
        } else {
            panic!("not a minter")
        }
    }

    pub fn get_minters(self) -> Vec<AccountId> {
        self.minters.clone()
    }

    /// sets metadata icon and/or reference
    #[payable]
    pub fn set_metadata(&mut self, svg_string: Option<String>, reference: Option<String>) {
        assert_one_yocto();
        self.assert_owner();
        let mut m = self.internal_get_ft_metadata();

        if let Some(s) = svg_string {
            m.icon = Some(s);
        }

        if let Some(r) = reference {
            m.reference = Some(r);
        }

        self.metadata.set(&m);
    }
}

#[near]
impl FungibleTokenCore for SpotToken {
  #[payable]
  fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
      assert_one_yocto();
      let sender_id = env::predecessor_account_id();
      let amount: Balance = amount.into();
      self.internal_transfer(&sender_id, &receiver_id, amount, memo);
  }

  #[payable]
  fn ft_transfer_call(
      &mut self,
      receiver_id: AccountId,
      amount: U128,
      memo: Option<String>,
      msg: String,
  ) -> PromiseOrValue<U128> {
      assert_one_yocto();
      require!(env::prepaid_gas() > GAS_FOR_FT_TRANSFER_CALL, "More gas is required");
      let sender_id = env::predecessor_account_id();
      let amount: Balance = amount.into();
      self.internal_transfer(&sender_id, &receiver_id, amount, memo);
      let receiver_gas = env::prepaid_gas()
          .checked_sub(GAS_FOR_FT_TRANSFER_CALL)
          .unwrap_or_else(|| env::panic_str("Prepaid gas overflow"));
      // Initiating receiver's call and the callback
      ext_ft_receiver::ext(receiver_id.clone())
          .with_static_gas(receiver_gas)
          .ft_on_transfer(sender_id.clone(), amount.into(), msg)
          .then(
              ext_ft_resolver::ext(env::current_account_id())
                  .with_static_gas(GAS_FOR_RESOLVE_TRANSFER)
                  .ft_resolve_transfer(sender_id, receiver_id, amount.into()),
          )
          .into()
  }

  fn ft_total_supply(&self) -> U128 {
      self.total_supply.into()
  }

  fn ft_balance_of(&self, account_id: AccountId) -> U128 {
      self.accounts.get(&account_id).unwrap_or(0).into()
  }
}

#[near]
impl FungibleTokenMetadataProvider for SpotToken {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}

#[near]
impl FungibleTokenResolver for SpotToken {
    #[private]
    fn ft_resolve_transfer(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128 {
        let (used_amount, burned_amount) = self.internal_ft_resolve_transfer(
          &sender_id,
          receiver_id,
          amount
        );

        if burned_amount > 0 {
            log!("Account @{} burned {}", sender_id, burned_amount);
        }

        used_amount.into()
    }
}
