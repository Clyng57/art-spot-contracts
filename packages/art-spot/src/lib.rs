
use near_sdk::{assert_one_yocto, env, near, AccountId, BorshStorageKey, Duration, Gas, NearToken, PanicOnDefault, Promise, PromiseResult, PublicKey};
use near_sdk::json_types::{U128};
use near_sdk::store::{LookupMap, Vector};
use as_sdk::{Id, Ownable, YoctoNear, Allowlist, *};

mod internal;
mod migrate;
mod update;
mod storage;
mod base32;
pub mod id;
pub mod user;
mod ctx;
mod storage_tracker;

use crate::user::*;

// const MIN_STORAGE: NearToken = NearToken::from_yoctonear(1_000_000_000_000_000_000_000); //0.001Ⓝ
// const INITIAL_BALANCE: NearToken = NearToken::from_yoctonear(3_000_000_000_000_000_000_000_000); // 3e24yN, 3Ⓝ

const STORAGE_COST_PER_BYTE: YoctoNear = YoctoNear(10_000_000_000_000_000_000u128); // 10e18yⓃ

// const TGAS: Gas = Gas::from_tgas(1); // 10e12yⓃ
// const NO_DEPOSIT: NearToken = NearToken::from_near(0); // 0yⓃ

/// Gas attached to the callback from account creation.
pub const ON_CREATE_ACCOUNT_CALLBACK_GAS: Gas = Gas::from_gas(13_000_000_000_000);

fn is_promise_success() -> bool {
  assert_eq!(
      env::promise_results_count(),
      1,
      "Contract expected a result on the callback"
  );
  match env::promise_result(0) {
      PromiseResult::Successful(_) => true,
      _ => false,
  }
}

#[near(serializers = [borsh])]
#[derive(BorshStorageKey)]
pub enum StorageKey {
    Users,
    Allowlist
}

#[near(contract_state)]
#[derive(PanicOnDefault, Ownable, Allowlist)]
pub struct ArtSpot {
    /// Owner of the contract
    pub owner: Id,
    pub staging_duration: Duration,
    pub staging_timestamp: Timestamp,
    /// The list of accounts that can create sub-accounts
    pub allowlist: Vector<Id>,
    /// Users of the contract
    pub users: LookupMap<Id, User>
}

#[near]
impl ArtSpot {
  #[init]
  #[private]
  pub fn new(owner_id: Id, staging_duration: Duration) -> Self {
    as_sdk::require!(!env::state_exists(), "Already initialized");

    let mut this = Self {
      owner: owner_id.clone(),
      staging_duration,
      staging_timestamp: Timestamp::from(0),
      allowlist: Vector::new(StorageKey::Allowlist),
      users: LookupMap::new(StorageKey::Users)
    };

    this.allowlist.push(owner_id);
    this
  }

  pub fn get_owner(&self) -> Id {
    self.owner.clone()
  }

  pub fn set_owner(&mut self, id: Id) {
    self.assert_owner();
    self.owner = id;
  }

  pub fn get_allowlist(&self) -> Vec<Id> {
    self.allowlist.into_iter().cloned().collect()
  }

  pub fn add_to_allowlist(&mut self, account_id: Id) {
    self.assert_owner();
    self.allowlist.push(account_id);
  }

  /// Create new account and deposit passed funds.
  #[payable]
  pub fn create_account(
      &mut self,
      id: String,
      public_key: String,
  ) -> Promise {
      self.assert_allowlisted();

      let account_id = Id::new(id);
      let public_key: PublicKey = public_key.parse().expect("Invalid public key");
      let amount: NearToken = env::attached_deposit();

      Promise::new(account_id.to_account_id())
          .create_account()
          .add_full_access_key(public_key.into())
          .transfer(amount)
          .then(
              Self::ext(env::current_account_id())
                  .with_static_gas(ON_CREATE_ACCOUNT_CALLBACK_GAS)
                  .on_account_created(
                      env::predecessor_account_id(),
                      amount.into()
                  )
          )
  }

  /// Callback after executing `create_account` or `create_account_advanced`.
  pub fn on_account_created(&mut self, predecessor_account_id: AccountId, amount: NearToken) -> bool {
      self.assert_self();
      let creation_succeeded = is_promise_success();

      if !creation_succeeded {
          // In case of failure, send funds back.
          Promise::new(predecessor_account_id).transfer(amount.into());
      }

      creation_succeeded
  }

  /// Create new account and deposit passed funds while deploying a contract.
  #[payable]
  pub fn create_contract(
      &mut self,
      prefix: String,
      code: Vec<u8>,
      public_key: Option<PublicKey>,
  ) -> Promise {
      self.assert_allowlisted();

      let current_account = env::current_account_id();
      let account_id: AccountId = format!("{prefix}.{current_account}").parse().unwrap();

      require!(
          env::is_valid_account_id(account_id.as_bytes()),
          "Invalid subaccount"
      );

      // Assert enough tokens are attached to create the account and deploy the contract
      let attached: NearToken = env::attached_deposit();
      let contract_bytes = code.clone().len() as u128;
      let minimum_needed: YoctoNear = env::storage_byte_cost().saturating_mul(contract_bytes);

      as_sdk::require!(
          attached >= minimum_needed,
          "Attach at least {minimum_needed} yⓃ"
      );

      let mut promise: Promise = Promise::new(account_id.clone())
          .create_account()
          .transfer(attached)
          .deploy_contract(code);

      // Add full access key is the user passes one
      if let Some(pk) = public_key {
          promise = promise.add_full_access_key(pk);
      }

      promise
        .then(
          Self::ext(env::current_account_id())
              .with_static_gas(ON_CREATE_ACCOUNT_CALLBACK_GAS)
              .on_account_created(
                  env::predecessor_account_id(),
                  attached.into()
              )
        ) // NM686
  }

  #[payable]
  pub fn update_account(
    &mut self,
    id: Id,
    email: Option<String>,
    name: Option<String>,
    bio: Option<String>,
    avatar: Option<String>,
    cover: Option<String>,
    roles: Option<Vec<String>>,
  ) {
    let mut user = self.internal_unwrap_account(&id);

    if let Some(email) = email {
      user.email = email;
    }

    if let Some(name) = name {
      user.name = name;
    }

    if let Some(bio) = bio {
      user.bio = Some(bio);
    }

    if let Some(avatar) = avatar {
      user.avatar = Some(avatar);
    }

    if let Some(cover) = cover {
      user.cover = Some(cover);
    }

    if let Some(roles) = roles {
      let new_roles: Vec<Role> = roles.into_iter().map(|role| {
        let new_role = Role::from(role);

        if new_role == Role::Admin {
          self.assert_allowlisted();
        }

        if new_role == Role::Moderator {
          self.assert_admin();
        }

        new_role
      }).collect();

      user.roles = new_roles;
    }

    user.storage_tracker.start();
    self.users.insert(id.clone(), user.clone());
    user.storage_tracker.stop();
    self.internal_set_account(&id, user);
  }
}
