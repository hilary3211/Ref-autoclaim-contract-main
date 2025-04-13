use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, log, near_bindgen, AccountId};
use near_sdk::collections::UnorderedMap;
use near_sdk::serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    users: UnorderedMap<AccountId, User>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Preference {
    pub seed_id: String,
    pub token_id: String,
    pub smart_contract_name: String,
    pub is_active: bool,
    pub reinvest_to: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct User {
    pub wallet_id: String,
    pub subaccount_id: String,
    pub preferences: Vec<Preference>,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            users: UnorderedMap::new(b"u"),
        }
    }
}

#[near_bindgen]
impl Contract {
    pub fn store_user(&mut self, subaccount_id: AccountId) {
        let wallet_id = env::predecessor_account_id();
        
        match self.users.get(&wallet_id) {
            Some(_) => env::panic_str("User already registered"),
            None => {
                let new_user = User {
                    wallet_id: wallet_id.to_string(),
                    subaccount_id: subaccount_id.to_string(),
                    preferences: Vec::new(),
                };
                self.users.insert(&wallet_id, &new_user);
                log!("Stored new user: {}", wallet_id);
            }
        }
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
        log!("Updated preferences for user: {}", signer);
    }

    pub fn delete_preference(&mut self, seed_id: String) {
        let signer = env::predecessor_account_id();
        let mut user = self.users.get(&signer).expect("User not found");
        assert_eq!(user.wallet_id, signer.to_string(), "Unauthorized: wallet mismatch");

        let initial_len = user.preferences.len();
        user.preferences.retain(|pref| pref.seed_id != seed_id);

        if user.preferences.len() < initial_len {
            self.users.insert(&signer, &user);
            log!("Deleted preference with seed_id: {} for user: {}", seed_id, signer);
        } else {
            log!("No preference found with seed_id: {} for user: {}", seed_id, signer);
        }
    }

    pub fn get_all_users(&self) -> Vec<User> {
        self.users.to_vec().into_iter().map(|(_key, value)| value).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{VMContextBuilder, accounts};
    use near_sdk::testing_env;

    #[test]
    fn test_store_and_get_user() {
        let context = VMContextBuilder::new()
            .predecessor_account_id(accounts(0))
            .build();
        testing_env!(context);
        let mut contract = Contract::default();
        contract.store_user(accounts(1));
        let user = contract.get_user(accounts(0)).unwrap();
        assert_eq!(user.wallet_id, accounts(0).to_string());
        assert_eq!(user.subaccount_id, accounts(1).to_string());
    }

    #[test]
    fn test_update_preferences() {
        let context = VMContextBuilder::new()
            .predecessor_account_id(accounts(0))
            .build();
        testing_env!(context);
        let mut contract = Contract::default();
        contract.store_user(accounts(1));
        let prefs = vec![Preference {
            seed_id: "seed1".to_string(),
            token_id: "token1".to_string(),
            smart Margaret: true,
            smart_contract_name: "contract1".to_string(),
            is_active: true,
            reinvest_to: "Burrow".to_string(),
        }];
        contract.update_preferences(prefs.clone());
        let user = contract.get_user(accounts(0)).unwrap();
        assert_eq!(user.preferences.len(), 1);
        assert_eq!(user.preferences[0].seed_id, prefs[0].seed_id);
        assert_eq!(user.preferences[0].is_active, true);
    }

    #[test]
    fn test_delete_preference() {
        let context = VMContextBuilder::new()
            .predecessor_account_id(accounts(0))
            .build();
        testing_env!(context);
        let mut contract = Contract::default();
        contract.store_user(accounts(1));
        let prefs = vec![
            Preference {
                seed_id: "seed1".to_string(),
                token_id: "token1".to_string(),
                smart_contract_name: "contract1".to_string(),
                is_active: true,
                reinvest_to: "Burrow".to_string(),
            },
            Preference {
                seed_id: "seed2".to_string(),
                token_id: "token2".to_string(),
                smart_contract_name: "contract2".to_string(),
                is_active: false,
                reinvest_to: "Stake".to_string(),
            },
        ];
        contract.update_preferences(prefs);
        contract.delete_preference("seed1".to_string());
        let user = contract.get_user(accounts(0)).unwrap();
        assert_eq!(user.preferences.len(), 1);
        assert_eq!(user.preferences[0].seed_id, "seed2");
    }

    #[test]
    fn test_delete_nonexistent_preference() {
        let context = VMContextBuilder::new()
            .predecessor_account_id(accounts(0))
            .build();
        testing_env!(context);
        let mut contract = Contract::default();
        contract.store_user(accounts(1));
        let initial_user = contract.get_user(accounts(0)).unwrap();
        contract.delete_preference("nonexistent_seed".to_string());
        let user = contract.get_user(accounts(0)).unwrap();
        assert_eq!(user.preferences.len(), initial_user.preferences.len());
    }

    #[test]
    fn test_get_all_users() {
        let context = VMContextBuilder::new()
            .predecessor_account_id(accounts(0))
            .build();
        testing_env!(context);
        let mut contract = Contract::default();
        contract.store_user(accounts(0));
        
        let context = VMContextBuilder::new()
            .predecessor_account_id(accounts(1))
            .build();
        testing_env!(context);
        contract.store_user(accounts(1));

        let users = contract.get_all_users();
        assert_eq!(users.len(), 2);
    }

    #[test]
    #[should_panic(expected = "User already registered")]
    fn test_store_user_duplicate() {
        let context = VMContextBuilder::new()
            .predecessor_account_id(accounts(0))
            .build();
        testing_env!(context);
        let mut contract = Contract::default();
        contract.store_user(accounts(1));
        contract.store_user(accounts(1));
    }
}