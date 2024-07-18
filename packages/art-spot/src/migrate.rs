
use crate::*;

#[near(serializers = [borsh])]
pub struct OldState {
  owner: Id,
  allowlist: Vector<Id>,
  users: LookupMap<Id, User>,
}

#[near]
impl ArtSpot {
  #[private]
  #[init(ignore_state)]
  pub fn migrate() -> Self {
    // retrieve the current state from the contract
    let old_state: OldState = env::state_read().expect("failed");

    Self {
      owner: old_state.owner,
      allowlist: old_state.allowlist,
      users: old_state.users,
    }
  }
}
