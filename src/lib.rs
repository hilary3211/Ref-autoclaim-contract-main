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
    pub username: String,
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
    pub fn store_user(&mut self, username: String, subaccount_id: String) {
        let wallet_id = env::predecessor_account_id();
        let new_user = User {
            username,
            wallet_id: wallet_id.to_string(),
            subaccount_id,
            preferences: Vec::new(),
        };
        self.users.insert(&wallet_id, &new_user);
        log!("Stored user: {}", wallet_id);
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
        contract.store_user("alice".to_string(), "alice.sub".to_string());
        let user = contract.get_user(accounts(0)).unwrap();
        assert_eq!(user.username, "alice");
        assert_eq!(user.wallet_id, accounts(0).to_string());
        assert_eq!(user.subaccount_id, "alice.sub");
    }

    #[test]
    fn test_update_preferences() {
        let context = VMContextBuilder::new()
            .predecessor_account_id(accounts(0))
            .build();
        testing_env!(context);
        let mut contract = Contract::default();
        contract.store_user("bob".to_string(), "bob.sub".to_string());
        let prefs = vec![Preference {
            seed_id: "seed1".to_string(),
            token_id: "token1".to_string(),
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
    fn test_get_all_users() {
        let context = VMContextBuilder::new()
            .predecessor_account_id(accounts(0))
            .build();
        testing_env!(context);
        let mut contract = Contract::default();
        contract.store_user("alice".to_string(), "alice.sub".to_string());
        
        let context = VMContextBuilder::new()
            .predecessor_account_id(accounts(1))
            .build();
        testing_env!(context);
        contract.store_user("bob".to_string(), "bob.sub".to_string());

        let users = contract.get_all_users();
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].username, "alice");
        assert_eq!(users[1].username, "bob");
    }
}