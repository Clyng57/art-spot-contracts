
use near_sdk::env;
use crate::*;

use crate::ctx;

impl ArtSpot {
  pub(crate) fn assert_owner(&self) {
    require!(
      ctx::predecessor_id() == self.owner,
      "Only owner can call this method"
    );
  }

  pub(crate) fn assert_self(&self) {
    require!(
      ctx::predecessor_id() == ctx::current_id(),
      "Callback can only be called from the contract"
    );
  }

  pub(crate) fn assert_allowlisted(&self) {
    let found_id = self.allowlist.iter().find(|account_id| *account_id == &ctx::predecessor_id());

    require!(
      found_id.is_some(),
      "Only allowlisted accounts can call this method"
    );
  }

  pub(crate) fn assert_role(&self, role: Role) {
    let user = crate::unwrap!(self.internal_get_account(&ctx::predecessor_id()));

    require!(
      user.roles.contains(&role),
      "Only accounts with the required role can call this method"
    );
  }

  pub(crate) fn assert_admin(&self) {
    self.assert_role(Role::Admin);
  }

  pub(crate) fn internal_get_account(&self, id: &Id) -> Option<User> {
    self.users.get(id).cloned()
  }

  pub(crate) fn internal_unwrap_account(&self, id: &Id) -> User {
    crate::unwrap!(self.internal_get_account(id))
  }
}
