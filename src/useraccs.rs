
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId};
use near_sdk::collections::UnorderedMap;
use near_sdk::serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use serde_json::json;

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
        }
    }
}

#[near_bindgen]
impl Contract {
    pub fn store_user(&mut self, username: String, subaccount_id: String) {
        let wallet_id = env::signer_account_id();
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
        let signer = env::signer_account_id();
        let mut user = self.users.get(&signer).expect("User not found");

        assert_eq!(user.wallet_id, signer.to_string(), "Unauthorized: wallet mismatch");

        // Append the new preferences to the existing preferences
        user.preferences.extend(prefs);

        // Update the user in the contract state
        self.users.insert(&signer, &user);
    }

    pub fn get_all_users(&self) -> Vec<User> {
        self.users.to_vec().into_iter().map(|(_key, value)| value).collect()
    }
}






