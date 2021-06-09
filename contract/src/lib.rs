/*
 * This is an example of a Rust smart contract with two simple, symmetric functions:
 *
 * 1. set_greeting: accepts a greeting, such as "howdy", and records it for the user (account_id)
 *    who sent the request
 * 2. get_greeting: accepts an account_id and returns the greeting saved for it, defaulting to
 *    "Hello"
 *
 * Learn more about writing NEAR smart contracts with Rust:
 * https://github.com/near/near-sdk-rs
 *
 */

// To conserve gas, efficient serialization is achieved through Borsh (http://borsh.io/)
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::wee_alloc;
use near_sdk::{env, near_bindgen, ext_contract, Promise, PromiseResult, Gas};
use near_sdk::json_types::{ValidAccountId, U128};
use std::collections::HashMap;
use near_sdk::serde_json::{self, json};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Structs in Rust are similar to other languages, and may include impl keyword as shown below
// Note: the names of the structs are not important when calling the smart contract, but the function names are
#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct Welcome {
    records: HashMap<String, String>,
}

const GAS_BASE_COMPUTE: Gas = 5_000_000_000_000;
/// Indicates there are no deposit for a callback for better readability.
const NO_DEPOSIT: u128 = 0;

#[ext_contract(ext_self)]
pub trait ExtDemo {
    /// Callback after receiving balances
    fn on_get_balance(&self) -> bool;
}

#[ext_contract(ext_fungible_token)]
pub trait FungibleTokenContract {
    /// Returns the balance of the account. If the account doesn't exist must returns `"0"`.
    fn ft_balance_of(&self, account_id: AccountId) -> U128;
}

fn get_promise_result() -> U128 {
    assert_eq!(
        env::promise_results_count(),
        1,
        "Contract expected a result on the callback"
    );
    match env::promise_result(0) {
        PromiseResult::Successful(x) => (serde_json::from_slice::<U128>(&x)).unwrap_or(U128(0)),
        _ => panic!("Promise was not successful")
    }
}

#[near_bindgen]
impl Welcome {
    pub fn get_ft_balance1(&self, contract_id: ValidAccountId, account_id: ValidAccountId) -> Promise {
        Promise::new(contract_id.as_ref().clone())
            .function_call(
                b"ft_balance_of".to_vec(), 
                serde_json::to_vec(&json!({"account_id": account_id.as_ref().clone()})).unwrap(), 
                NO_DEPOSIT, 
                GAS_BASE_COMPUTE)
            .then(ext_self::on_get_balance(
                &env::current_account_id(),
                NO_DEPOSIT,
                GAS_BASE_COMPUTE,
            ))
    }

    pub fn get_ft_balance2(&self, contract_id: ValidAccountId, account_id: ValidAccountId) -> Promise {
        ext_fungible_token::ft_balance_of(
            account_id.as_ref().clone(),
            contract_id.as_ref(),
            NO_DEPOSIT,
            GAS_BASE_COMPUTE
        ).then(ext_self::on_get_balance(
            &env::current_account_id(),
            NO_DEPOSIT,
            GAS_BASE_COMPUTE,
        ))
    }

    pub fn on_get_balance(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Callback can only be called from the contract"
        );
        let balance = get_promise_result();
        env::log(format!("The received balance is {}", balance.0).as_bytes());
    }
}

/*
 * The rest of this file holds the inline tests for the code above
 * Learn more about Rust tests: https://doc.rust-lang.org/book/ch11-01-writing-tests.html
 *
 * To run from contract directory:
 * cargo test -- --nocapture
 *
 * From project root, to run in combination with frontend tests:
 * yarn test
 *
 */
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};

    // mock the context for testing, notice "signer_account_id" that was accessed above from env::
    fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
        VMContext {
            current_account_id: "alice_near".to_string(),
            signer_account_id: "bob_near".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: "carol_near".to_string(),
            input,
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
            epoch_height: 19,
        }
    }
}
