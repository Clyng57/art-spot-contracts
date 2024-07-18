
use std::sync::Mutex;

use near_sdk::StorageUsage;

use crate::{env, utils, Balance, Id};

static mut STORAGE_TRACKING: bool = false;
static mut INITIAL_STORAGE_USED: Mutex<StorageUsage> = Mutex::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StorageHistory {
  pub bytes_added: StorageUsage,
  pub bytes_released: StorageUsage,
}

impl StorageHistory {
  pub fn new(bytes_added: StorageUsage, bytes_released: StorageUsage) -> Self {
    Self {
      bytes_added,
      bytes_released,
    }
  }
}

impl Default for StorageHistory {
  fn default() -> Self {
    Self {
      bytes_added: 0,
      bytes_released: 0,
    }
  }
}

pub(crate) fn is_storage_tracking() -> bool {
  unsafe { STORAGE_TRACKING.clone() }
}

pub(crate) fn start_storage_tracking() {
  unsafe {
    utils::require(!STORAGE_TRACKING, "Storage tracking is already started");
    STORAGE_TRACKING = true;
    let mut initial_storage_used = INITIAL_STORAGE_USED.lock().expect("Lock storage usage");
    *initial_storage_used = env::storage_usage();
  }
}

pub(crate) fn stop_storage_tracking(previous_history: Option<StorageHistory>) -> StorageHistory {
  unsafe {
    STORAGE_TRACKING = false;
    let mut initial_storage_used = INITIAL_STORAGE_USED.lock().expect("Lock storage usage");
    let storage_used: StorageUsage = env::storage_usage();

    let mut storage_history: StorageHistory = previous_history.unwrap_or_default();

    if storage_used >= *initial_storage_used {
      storage_history.bytes_added += storage_used - *initial_storage_used;
    } else {
      storage_history.bytes_released += *initial_storage_used - storage_used;
    }

    *initial_storage_used = 0;
    storage_history
  }
}

pub fn current_id() -> Id {
  Id::new(crate::env::current_account_id().as_str())
}

pub fn signer_id() -> Id {
  Id::new(env::signer_account_id().as_str())
}

pub fn predecessor_id() -> Id {
  Id::new(env::predecessor_account_id().as_str())
}

pub fn balance() -> Balance {
  env::account_balance().as_yoctonear()
}

pub fn locked_balance() -> Balance {
  env::account_locked_balance().as_yoctonear()
}

pub fn attached_deposit() -> Balance {
  env::attached_deposit().as_yoctonear()
}

pub fn storage_usage() -> StorageUsage {
  env::storage_usage()
}
