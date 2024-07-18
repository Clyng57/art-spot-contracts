use near_sdk::json_types::U128;
use near_workspaces::{types::NearToken, Account, AccountId, Contract, DevNetwork, Worker};
use tokio::{fs::File, io::{self, AsyncReadExt}};

const ONE_YOCTO: NearToken = NearToken::from_yoctonear(1);

async fn register_user(contract: &Contract, account_id: &AccountId) -> anyhow::Result<()> {
    let res = contract
        .call("storage_deposit")
        .args_json((account_id, Option::<bool>::None))
        .max_gas()
        .deposit(near_sdk::env::storage_byte_cost().saturating_mul(125))
        .transact()
        .await?;
    assert!(res.is_success());

    Ok(())
}

async fn read_file(path: &str) -> io::Result<Vec<u8>> {
  let mut f = File::open(path).await?;
  let mut buffer: Vec<u8> = Vec::new();

  // read the whole file
  f.read_to_end(&mut buffer).await?;
  Ok(buffer)
}

async fn init(
    worker: &Worker<impl DevNetwork>,
    initial_balance: U128,
) -> anyhow::Result<(Contract, Account)> {
  let contract_wasm = read_file("../../compiled/spot_token.wasm").await?;
    let ft_contract = worker.dev_deploy(&contract_wasm).await?;

    let res = ft_contract
        .call("new_default_meta")
        .args_json((ft_contract.id(), initial_balance))
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());

    let alice = ft_contract
        .as_account()
        .create_subaccount("alice")
        .initial_balance(NearToken::from_near(10))
        .transact()
        .await?
        .into_result()?;
    register_user(&ft_contract, alice.id()).await?;

    let res = ft_contract
        .call("storage_deposit")
        .args_json((alice.id(), Option::<bool>::None))
        .deposit(near_sdk::env::storage_byte_cost().saturating_mul(125))
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());

    return Ok((ft_contract, alice));
}

#[tokio::test]
async fn test_total_supply() -> anyhow::Result<()> {
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());
    let worker = near_workspaces::sandbox().await?;
    let (contract, _) = init(&worker, initial_balance).await?;

    let res = contract.call("ft_total_supply").view().await?;
    assert_eq!(res.json::<U128>()?, initial_balance);

    Ok(())
}

#[tokio::test]
async fn test_storage_deposit_not_enough_deposit() -> anyhow::Result<()> {
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());
    let worker = near_workspaces::sandbox().await?;
    let (contract, _) = init(&worker, initial_balance).await?;

    let new_account = contract
        .as_account()
        .create_subaccount("new-account")
        .initial_balance(NearToken::from_near(10))
        .transact()
        .await?
        .into_result()?;

    let new_account_balance_before_deposit = new_account.view_account().await?.balance;
    let contract_balance_before_deposit = contract.view_account().await?.balance;

    let minimal_deposit = near_sdk::env::storage_byte_cost().saturating_mul(125);
    let res = new_account
        .call(contract.id(), "storage_deposit")
        .args(b"{}".to_vec())
        .max_gas()
        .deposit(minimal_deposit.saturating_sub(NearToken::from_yoctonear(1)))
        .transact()
        .await?;
    assert!(res.is_failure());

    let new_account_balance_diff = new_account_balance_before_deposit
        .saturating_sub(new_account.view_account().await?.balance);
    // new_account is charged the transaction fee, so it should loose some NEAR
    assert!(new_account_balance_diff > NearToken::from_near(0));
    assert!(new_account_balance_diff < NearToken::from_millinear(1));

    let contract_balance_diff =
        contract.view_account().await?.balance.saturating_sub(contract_balance_before_deposit);
    // contract receives a gas rewards for the function call, so it should gain some NEAR
    assert!(contract_balance_diff > NearToken::from_near(0));
    assert!(contract_balance_diff < NearToken::from_yoctonear(30_000_000_000_000_000_000));

    Ok(())
}

#[tokio::test]
async fn test_storage_deposit_minimal_deposit() -> anyhow::Result<()> {
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());
    let worker = near_workspaces::sandbox().await?;
    let (contract, _) = init(&worker, initial_balance).await?;

    let new_account = contract
        .as_account()
        .create_subaccount("new-account")
        .initial_balance(NearToken::from_near(10))
        .transact()
        .await?
        .into_result()?;

    let new_account_balance_before_deposit = new_account.view_account().await?.balance;
    let contract_balance_before_deposit = contract.view_account().await?.balance;

    let minimal_deposit = near_sdk::env::storage_byte_cost().saturating_mul(125);
    new_account
        .call(contract.id(), "storage_deposit")
        .args(b"{}".to_vec())
        .max_gas()
        .deposit(minimal_deposit)
        .transact()
        .await?
        .into_result()?;

    let new_account_balance_diff = new_account_balance_before_deposit
        .saturating_sub(new_account.view_account().await?.balance);
    // new_account is charged the transaction fee, so it should loose a bit more than minimal_deposit
    assert!(new_account_balance_diff > minimal_deposit);
    assert!(
        new_account_balance_diff < minimal_deposit.saturating_add(NearToken::from_millinear(1))
    );

    let contract_balance_diff =
        contract.view_account().await?.balance.saturating_sub(contract_balance_before_deposit);
    // contract receives a gas rewards for the function call, so the difference should be slightly more than minimal_deposit
    assert!(contract_balance_diff > minimal_deposit);
    assert!(
        contract_balance_diff
            < minimal_deposit.saturating_add(NearToken::from_yoctonear(30_000_000_000_000_000_000))
    );

    Ok(())
}

#[tokio::test]
async fn test_storage_deposit_refunds_excessive_deposit() -> anyhow::Result<()> {
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());
    let worker = near_workspaces::sandbox().await?;
    let (contract, _) = init(&worker, initial_balance).await?;

    let minimal_deposit = near_sdk::env::storage_byte_cost().saturating_mul(125);

    // Check the storage balance bounds to make sure we have the right minimal deposit
    //
    #[derive(near_sdk::serde::Serialize, near_sdk::serde::Deserialize)]
    #[serde(crate = "near_sdk::serde")]
    struct StorageBalanceBounds {
        min: U128,
        max: U128,
    }
    let storage_balance_bounds: StorageBalanceBounds =
        contract.call("storage_balance_bounds").view().await?.json()?;
    assert_eq!(storage_balance_bounds.min, minimal_deposit.as_yoctonear().into());
    assert_eq!(storage_balance_bounds.max, minimal_deposit.as_yoctonear().into());

    // Check that a non-registered account does not have storage balance
    //
    #[derive(near_sdk::serde::Serialize, near_sdk::serde::Deserialize)]
    #[serde(crate = "near_sdk::serde")]
    struct StorageBalanceOf {
        total: U128,
        available: U128,
    }
    let storage_balance_bounds: Option<StorageBalanceOf> = contract
        .call("storage_balance_of")
        .args_json(near_sdk::serde_json::json!({"account_id": "non-registered-account"}))
        .view()
        .await?
        .json()?;
    assert!(storage_balance_bounds.is_none());

    // Create a new account and deposit some NEAR to cover the storage
    //
    let new_account = contract
        .as_account()
        .create_subaccount("new-account")
        .initial_balance(NearToken::from_near(10))
        .transact()
        .await?
        .into_result()?;

    let new_account_balance_before_deposit = new_account.view_account().await?.balance;
    let contract_balance_before_deposit = contract.view_account().await?.balance;

    new_account
        .call(contract.id(), "storage_deposit")
        .args(b"{}".to_vec())
        .max_gas()
        .deposit(NearToken::from_near(5))
        .transact()
        .await?
        .into_result()?;

    // The expected storage balance should be the minimal deposit,
    // the balance of the account should be reduced by the deposit,
    // and the contract should gain the deposit.
    //
    let storage_balance_bounds: StorageBalanceOf = contract
        .call("storage_balance_of")
        .args_json(near_sdk::serde_json::json!({"account_id": new_account.id()}))
        .view()
        .await?
        .json()?;
    assert_eq!(storage_balance_bounds.total, minimal_deposit.as_yoctonear().into());
    assert_eq!(storage_balance_bounds.available, 0.into());

    let new_account_balance_diff = new_account_balance_before_deposit
        .saturating_sub(new_account.view_account().await?.balance);
    // new_account is charged the transaction fee, so it should loose a bit more than minimal_deposit
    assert!(new_account_balance_diff > minimal_deposit);
    assert!(
        new_account_balance_diff < minimal_deposit.saturating_add(NearToken::from_millinear(1))
    );

    let contract_balance_diff =
        contract.view_account().await?.balance.saturating_sub(contract_balance_before_deposit);
    // contract receives a gas rewards for the function call, so the difference should be slightly more than minimal_deposit
    assert!(contract_balance_diff > minimal_deposit);
    assert!(
        contract_balance_diff
            < minimal_deposit.saturating_add(NearToken::from_yoctonear(30_000_000_000_000_000_000))
    );

    Ok(())
}

#[tokio::test]
async fn test_simple_transfer() -> anyhow::Result<()> {
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());
    let transfer_amount = U128::from(NearToken::from_near(100).as_yoctonear());
    let worker = near_workspaces::sandbox().await?;
    let (contract, alice) = init(&worker, initial_balance).await?;

    let res = contract
        .call("ft_transfer")
        .args_json((alice.id(), transfer_amount, Option::<bool>::None))
        .max_gas()
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
    assert!(res.is_success());

    let root_balance =
        contract.call("ft_balance_of").args_json((contract.id(),)).view().await?.json::<U128>()?;
    let alice_balance =
        contract.call("ft_balance_of").args_json((alice.id(),)).view().await?.json::<U128>()?;
    assert_eq!(initial_balance.0 - transfer_amount.0, root_balance.0);
    assert_eq!(transfer_amount.0, alice_balance.0);

    Ok(())
}

#[tokio::test]
async fn test_close_account_empty_balance() -> anyhow::Result<()> {
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());
    let worker = near_workspaces::sandbox().await?;
    let (contract, alice) = init(&worker, initial_balance).await?;

    let res = alice
        .call(contract.id(), "storage_unregister")
        .args_json((Option::<bool>::None,))
        .max_gas()
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
    assert!(res.json::<bool>()?);

    Ok(())
}

#[tokio::test]
async fn test_close_account_non_empty_balance() -> anyhow::Result<()> {
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());
    let worker = near_workspaces::sandbox().await?;
    let (contract, _) = init(&worker, initial_balance).await?;

    let res = contract
        .call("storage_unregister")
        .args_json((Option::<bool>::None,))
        .max_gas()
        .deposit(ONE_YOCTO)
        .transact()
        .await;
    assert!(format!("{:?}", res)
        .contains("Can't unregister the account with the positive balance without force"));

    let res = contract
        .call("storage_unregister")
        .args_json((Some(false),))
        .max_gas()
        .deposit(ONE_YOCTO)
        .transact()
        .await;
    assert!(format!("{:?}", res)
        .contains("Can't unregister the account with the positive balance without force"));

    Ok(())
}

#[tokio::test]
async fn simulate_close_account_force_non_empty_balance() -> anyhow::Result<()> {
    let initial_balance = U128::from(NearToken::from_near(10000).as_yoctonear());
    let worker = near_workspaces::sandbox().await?;
    let (contract, _) = init(&worker, initial_balance).await?;

    let res = contract
        .call("storage_unregister")
        .args_json((Some(true),))
        .max_gas()
        .deposit(ONE_YOCTO)
        .transact()
        .await?;
    assert!(res.is_success());

    let res = contract.call("ft_total_supply").view().await?;
    assert_eq!(res.json::<U128>()?.0, 0);

    Ok(())
}
