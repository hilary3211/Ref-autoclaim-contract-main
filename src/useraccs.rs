use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, log, near_bindgen, AccountId, Promise, Timestamp, Gas, NearToken, PromiseResult, PromiseOrValue};
use near_sdk::collections::UnorderedMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::json_types::U128;
use schemars::JsonSchema;
use serde_json::json;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    users: UnorderedMap<AccountId, User>,
    last_compound_call: Timestamp, 
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Preference {
    pub seed_id: String,
    pub token_id: String,
    pub smart_contract_name: String,
    pub is_active: String,
    pub reinvest_to: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct User {
    pub username: String,
    pub wallet_id: String,
    pub subaccount_id: String,
    pub preferences: Vec<Preference>,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            users: UnorderedMap::new(b"u".to_vec()),
            last_compound_call: 0,
        }
    }
}

#[near_bindgen]
impl Contract {
    pub fn store_user(&mut self, username: String, subaccount_id: String) {
        let wallet_id = env::predecessor_account_id();
        let new_user = User {
            username,
            wallet_id: wallet_id.to_string(),
            subaccount_id,
            preferences: Vec::new(),
        };

        self.users.insert(&wallet_id, &new_user);
    }

    pub fn get_user(&self, wallet_id: AccountId) -> Option<User> {
        self.users.get(&wallet_id)
    }

    pub fn update_preferences(&mut self, prefs: Vec<Preference>) {
        let signer = env::predecessor_account_id();
        let mut user = self.users.get(&signer).expect("User not found");

        assert_eq!(user.wallet_id, signer.to_string(), "Unauthorized: wallet mismatch");

        user.preferences.extend(prefs);
        self.users.insert(&signer, &user);
    }

    pub fn get_all_users(&self) -> Vec<User> {
        self.users.to_vec().into_iter().map(|(_key, value)| value).collect()
    }


    pub fn compound(&mut self) -> Vec<Promise> {
        let current_time = env::block_timestamp();
        assert!(
            current_time - self.last_compound_call >= 3_600_000_000_000, 
            "Compound can only be called once every hour"
        );

        self.last_compound_call = current_time;
    
        let users = self.get_all_users();
        let mut promises = Vec::new();
    
        let pre_claim_balance = env::account_balance().as_yoctonear();
    
        for user in users {
            for pref in user.preferences {
                if pref.is_active == "true" {
             
                    if let Ok(contract_name) = pref.smart_contract_name.parse::<AccountId>() {
                       
                        let claim_all_rewards_promise = Promise::new(contract_name.clone())
                            .function_call(
                                "claim_all_rewards".to_string(),
                                json!({
                                    "seed_id": pref.seed_id,
                                    "neargas": 50,
                                    "tokenid": pref.token_id,
                                })
                                .to_string()
                                .into_bytes(),
                                NearToken::from_yoctonear(0),
                                Gas::from_tgas(130),
                            )
                            .then(
                                Promise::new(env::current_account_id())
                                    .function_call(
                                        "handle_claim_result".to_string(),
                                        json!({
                                            "contract_name": contract_name.to_string(),
                                            "action": "claim_all_rewards"
                                        })
                                        .to_string()
                                        .into_bytes(),
                                        NearToken::from_yoctonear(0),
                                        Gas::from_tgas(10),
                                    )
                            );
    
                      
                        let claim_from_burrow_promise = Promise::new(contract_name.clone())
                            .function_call(
                                "claim_from_burrow".to_string(),
                                json!({
                                    "neargas": 50,
                                })
                                .to_string()
                                .into_bytes(),
                                NearToken::from_yoctonear(0),
                                Gas::from_tgas(130),
                            )
                            .then(
                                Promise::new(env::current_account_id())
                                    .function_call(
                                        "handle_claim_result".to_string(),
                                        json!({
                                            "contract_name": contract_name.to_string(),
                                            "action": "claim_from_burrow"
                                        })
                                        .to_string()
                                        .into_bytes(),
                                        NearToken::from_yoctonear(0),
                                        Gas::from_tgas(10),
                                    )
                            );
    
                        promises.push(claim_all_rewards_promise);
                        promises.push(claim_from_burrow_promise);
                    } else {
                        log!("{}", format!(
                            "Failed to parse smart_contract_name: {}",
                            pref.smart_contract_name
                        ));
                    }
                }
            }
        }
    
        let post_claim_balance = env::account_balance().as_yoctonear();
        let balance_increase = post_claim_balance - pre_claim_balance;
    
        if balance_increase > 2_000_000_000_000_000_000_000_000 {
            let caller = env::predecessor_account_id();
            let caller_share = balance_increase * 5 / 100;
            let transfer_promise = Promise::new(caller)
                .transfer(NearToken::from_yoctonear(caller_share));
            promises.push(transfer_promise);
        }
    
        promises
    }




   #[private]
    pub fn handle_claim_result(&self, contract_name: String, action: String) -> PromiseOrValue<()> {
        if env::promise_results_count() > 0 {
            match env::promise_result(0) {
                near_sdk::PromiseResult::Successful(_) => {
                    log!("{}", format!("{} succeeded for {}", action, contract_name));
                }
                near_sdk::PromiseResult::Failed => {
                    log!("{}", format!(
                        "Failed to execute {} on {}: Account may not be registered",
                        action, contract_name
                    ));
                }
            }
        } else {
            log!("No promise result available");
        }
        PromiseOrValue::Value(())
    }




    pub fn reinvest(&mut self, minamountout: String, minamountout2: String) -> Vec<Promise> {
        let users = self.get_all_users();
        let mut promises = Vec::new();
    
        let balance = env::account_balance().as_yoctonear();
        let two_near = 2_000_000_000_000_000_000_000_000;
    
        if balance <= two_near {
            log!("Insufficient balance for reinvestment: {}", balance);
            return promises; // Early return if not enough balance
        }
    
        let stake_bal = balance - two_near; // Calculate once
    
        for user in users {
            for pref in user.preferences {
                if pref.is_active == "true" {
                    if let Ok(contract_name) = pref.smart_contract_name.parse::<AccountId>() {
                        if pref.reinvest_to == "Burrow" {
                            // Deposit into Burrow with attached NEAR
                            let deposit_promise = Promise::new(contract_name.clone())
                                .function_call(
                                    "deposit_into_burrow".to_string(),
                                    json!({
                                        "deposit_amount": stake_bal.to_string(),
                                        "neargas": 50,
                                    })
                                    .to_string()
                                    .into_bytes(),
                                    NearToken::from_yoctonear(stake_bal), // Attach the amount
                                    Gas::from_tgas(120), // Increased gas
                                )
                                .then(
                                    Promise::new(env::current_account_id())
                                        .function_call(
                                            "handle_reinvest_result".to_string(),
                                            json!({
                                                "contract_name": contract_name.to_string(),
                                                "action": "deposit_into_burrow"
                                            })
                                            .to_string()
                                            .into_bytes(),
                                            NearToken::from_yoctonear(0),
                                            Gas::from_tgas(10),
                                        )
                                );
    
                            promises.push(deposit_promise);
                        } else if pref.reinvest_to == "Stake" {
                            // Stake via Ref Finance
                            let swap_promise = Promise::new(contract_name.clone())
                                .function_call(
                                    "stake_xRef".to_string(),
                                    json!({
                                        "smart_contract_name": pref.smart_contract_name,
                                        "deposit_amount": stake_bal.to_string(),
                                        "neargas": 50,
                                        "receiver_id": "xtoken.ref-finance.near",
                                        "min_amount_out": minamountout,
                                        "pool_id": minamountout2,
                                    })
                                    .to_string()
                                    .into_bytes(),
                                    NearToken::from_yoctonear(stake_bal), // Attach the amount
                                    Gas::from_tgas(120), // Increased gas
                                )
                                .then(
                                    Promise::new(env::current_account_id())
                                        .function_call(
                                            "handle_reinvest_result".to_string(),
                                            json!({
                                                "contract_name": contract_name.to_string(),
                                                "action": "stake_xRef"
                                            })
                                            .to_string()
                                            .into_bytes(),
                                            NearToken::from_yoctonear(0),
                                            Gas::from_tgas(10),
                                        )
                                );
    
                            promises.push(swap_promise);
                        }
                    } else {
                        log!("{}", format!(
                            "Failed to parse smart_contract_name: {}",
                            pref.smart_contract_name
                        ));
                    }
                }
            }
        }
    
        promises
    }


    #[private]
    pub fn handle_reinvest_result(&self, contract_name: String, action: String) -> PromiseOrValue<()> {
        if env::promise_results_count() > 0 {
            match env::promise_result(0) {
                near_sdk::PromiseResult::Successful(_) => {
                    log!("{}", format!("{} succeeded for {}", action, contract_name));
                }
                near_sdk::PromiseResult::Failed => {
                    log!("{}", format!(
                        "Failed to execute {} on {}: Check account registration or balance",
                        action, contract_name
                    ));
                }
            }
        } else {
            log!("No promise result available");
        }
        PromiseOrValue::Value(())
    }


}
