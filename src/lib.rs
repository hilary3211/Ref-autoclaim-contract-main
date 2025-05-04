use near_sdk::borsh::{ self, BorshDeserialize, BorshSerialize };
use near_sdk::{ env, log, near_bindgen, AccountId };
use near_sdk::collections::UnorderedMap;
use near_sdk::serde::{ Deserialize, Serialize };
use schemars::JsonSchema;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    users: UnorderedMap<AccountId, User>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum ReinvestOption {
    #[schemars(with = "String")] Burrow {
        seed_id: String,
        token_id: AccountId,
    },
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum InvestOption {
    #[schemars(with = "String")] Burrow {
        seed_id: String,
        token_id: AccountId,
    },
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Preference {
    #[schemars(with = "String")]
    pub smart_contract_name: AccountId,
    pub is_active: bool,
    pub invested_in: InvestOption,
    pub reinvest_to: ReinvestOption,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct User {
    #[schemars(with = "String")]
    pub wallet_id: AccountId,
    #[schemars(with = "String")]
    pub subaccount_id: AccountId,
    pub preference: Option<Preference>,
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
                    wallet_id: wallet_id.clone(),
                    subaccount_id,
                    preference: None,
                };
                self.users.insert(&wallet_id, &new_user);
                log!("Stored new user: {}", wallet_id);
            }
        }
    }

    pub fn get_user(&self, wallet_id: AccountId) -> Option<User> {
        self.users.get(&wallet_id)
    }

    pub fn update_preference(&mut self, preference: Preference) {
        let signer = env::predecessor_account_id();
        let mut user = self.users.get(&signer).expect("User not found");
        assert_eq!(user.wallet_id, signer, "Unauthorized: wallet mismatch");

        match &preference.invested_in {
            InvestOption::Burrow { seed_id, .. } => {
                assert!(seed_id.len() <= 64, "seed_id must be 64 characters or less");
            }
        }
        match &preference.reinvest_to {
            ReinvestOption::Burrow { seed_id, .. } => {
                assert!(seed_id.len() <= 64, "seed_id must be 64 characters or less");
            }
        }

        user.preference = Some(preference);
        self.users.insert(&signer, &user);
        log!("Updated preference for user: {}", signer);
    }

    pub fn delete_preference(&mut self) {
        let signer = env::predecessor_account_id();
        let mut user = self.users.get(&signer).expect("User not found");
        assert_eq!(user.wallet_id, signer, "Unauthorized: wallet mismatch");

        if user.preference.is_some() {
            user.preference = None;
            self.users.insert(&signer, &user);
            log!("Deleted preference for user: {}", signer);
        } else {
            log!("No preference found for user: {}", signer);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{ VMContextBuilder, accounts };
    use near_sdk::testing_env;

    #[test]
    fn test_store_and_get_user() {
        let context = VMContextBuilder::new().predecessor_account_id(accounts(0)).build();
        testing_env!(context);
        let mut contract = Contract::default();
        contract.store_user(accounts(1));
        let user = contract.get_user(accounts(0)).unwrap();
        assert_eq!(user.wallet_id, accounts(0));
        assert_eq!(user.subaccount_id, accounts(1));
        assert!(user.preference.is_none());
    }

    #[test]
    fn test_update_preference() {
        let context = VMContextBuilder::new().predecessor_account_id(accounts(0)).build();
        testing_env!(context);
        let mut contract = Contract::default();
        contract.store_user(accounts(1));
        let preference = Preference {
            smart_contract_name: accounts(2),
            is_active: true,
            invested_in: InvestOption::Burrow {
                seed_id: "seed1".to_string(),
                token_id: accounts(3),
            },
            reinvest_to: ReinvestOption::Burrow {
                seed_id: "seed1".to_string(),
                token_id: accounts(3),
            },
        };
        contract.update_preference(preference.clone());
        let user = contract.get_user(accounts(0)).unwrap();
        let stored_pref = user.preference.unwrap();
        assert_eq!(stored_pref.is_active, preference.is_active);
        match stored_pref.invested_in {
            InvestOption::Burrow { seed_id, .. } => {
                assert_eq!(seed_id, "seed1");
            }
        }
    }

    #[test]
    fn test_delete_preference() {
        let context = VMContextBuilder::new().predecessor_account_id(accounts(0)).build();
        testing_env!(context);
        let mut contract = Contract::default();
        contract.store_user(accounts(1));
        let preference = Preference {
            smart_contract_name: accounts(2),
            is_active: true,
            invested_in: InvestOption::Burrow {
                seed_id: "seed1".to_string(),
                token_id: accounts(3),
            },
            reinvest_to: ReinvestOption::Burrow {
                seed_id: "seed1".to_string(),
                token_id: accounts(3),
            },
        };
        contract.update_preference(preference);
        contract.delete_preference();
        let user = contract.get_user(accounts(0)).unwrap();
        assert!(user.preference.is_none());
    }

    #[test]
    fn test_delete_nonexistent_preference() {
        let context = VMContextBuilder::new().predecessor_account_id(accounts(0)).build();
        testing_env!(context);
        let mut contract = Contract::default();
        contract.store_user(accounts(1));
        contract.delete_preference();
        let user = contract.get_user(accounts(0)).unwrap();
        assert!(user.preference.is_none());
    }

    #[test]
    #[should_panic(expected = "User already registered")]
    fn test_store_user_duplicate() {
        let context = VMContextBuilder::new().predecessor_account_id(accounts(0)).build();
        testing_env!(context);
        let mut contract = Contract::default();
        contract.store_user(accounts(1));
        contract.store_user(accounts(1));
    }
}




