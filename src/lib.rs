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

/// A method to provide a default instance of the Contract struct.
/// This function:
/// - Takes no parameters,
/// - Initializes an UnorderedMap for users with a unique prefix 'u',
/// - Returns a new Contract instance with the initialized users map.
impl Default for Contract {
    fn default() -> Self {
        Self {
            users: UnorderedMap::new(b"u"), 
        }
    }
}

#[near_bindgen]
impl Contract {
    /// A method to store a new user in the contract’s users map.
    /// You must supply:
    /// - username: The String representing the user’s name,
    /// - subaccount_id: The String representing the user’s subaccount ID.
    /// This function:
    /// - Uses the caller’s account ID (predecessor_account_id) as the wallet_id,
    /// - Creates a new User struct with the provided username, wallet_id, subaccount_id, and an empty preferences vector,
    /// - Inserts the user into the users map,
    /// - Logs the storage action,
    /// - Returns nothing (implicitly updates the contract state).
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

    /// A method to retrieve a user’s data from the contract by their wallet ID.
    /// You must supply:
    /// - wallet_id: The AccountId of the user to retrieve.
    /// This function:
    /// - Queries the users map for the given wallet_id,
    /// - Returns an Option<User> containing the user’s data if found, or None if not found.
    pub fn get_user(&self, wallet_id: AccountId) -> Option<User> {
        self.users.get(&wallet_id)
    }

    /// A method to update a user’s preferences by adding new preferences to their existing list.
    /// You must supply:
    /// - prefs: A Vec<Preference> containing the new preferences to add.
    /// This function:
    /// - Uses the caller’s account ID (predecessor_account_id) as the signer,
    /// - Retrieves the user’s data and panics if not found,
    /// - Verifies the signer matches the user’s wallet_id, panicking if unauthorized,
    /// - Appends the new preferences to the user’s existing preferences,
    /// - Updates the user in the users map,
    /// - Logs the update action,
    /// - Returns nothing (implicitly updates the contract state).
    pub fn update_preferences(&mut self, prefs: Vec<Preference>) {
        let signer = env::predecessor_account_id();
        let mut user = self.users.get(&signer).expect("User not found");
        assert_eq!(user.wallet_id, signer.to_string(), "Unauthorized: wallet mismatch");
        user.preferences.extend(prefs);
        self.users.insert(&signer, &user);
        log!("Updated preferences for user: {}", signer);
    }

    /// A method to retrieve all users stored in the contract.
    /// This function:
    /// - Takes no parameters (uses self),
    /// - Converts the users map to a vector of key-value pairs,
    /// - Maps the vector to extract only the User values (discarding keys),
    /// - Returns a Vec<User> containing all user data.
    pub fn get_all_users(&self) -> Vec<User> {
        self.users.to_vec().into_iter().map(|(_key, value)| value).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{VMContextBuilder, accounts};
    use near_sdk::testing_env;

    /// A test to verify that storing and retrieving a user works correctly.
    /// This test:
    /// - Sets up a VM context with accounts(0) as the predecessor,
    /// - Initializes a default contract,
    /// - Stores a user with username "alice" and subaccount "alice.sub",
    /// - Retrieves the user by wallet_id (accounts(0)),
    /// - Asserts that the retrieved user’s username, wallet_id, and subaccount_id match the expected values.
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

    /// A test to verify that updating a user’s preferences works correctly.
    /// This test:
    /// - Sets up a VM context with accounts(0) as the predecessor,
    /// - Initializes a default contract,
    /// - Stores a user with username "bob" and subaccount "bob.sub",
    /// - Creates a vector with one Preference and updates the user’s preferences,
    /// - Retrieves the user by wallet_id (accounts(0)),
    /// - Asserts that the user’s preferences vector has one entry and matches the provided preference.
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

    /// A test to verify that retrieving all users works correctly.
    /// This test:
    /// - Sets up a VM context with accounts(0) as the predecessor and stores a user "alice",
    /// - Changes the context to accounts(1) as the predecessor and stores a user "bob",
    /// - Retrieves all users from the contract,
    /// - Asserts that the returned vector contains two users with usernames "alice" and "bob".
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