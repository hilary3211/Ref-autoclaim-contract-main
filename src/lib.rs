use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, log, near_bindgen, AccountId};
use near_sdk::collections::UnorderedMap; 
use near_sdk::serde::{Deserialize, Serialize};

// Removed schemars::JsonSchema since AccountId doesn’t implement it natively
// If JSON schema is critical, we’d need a custom implementation or revert to String

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    users: UnorderedMap<AccountId, User>,
}

/// An enum to represent the possible reinvestment options for a preference.
/// Variants:
/// - Burrow: Indicates reinvestment into the Burrow platform,
/// - Stake: Indicates reinvestment into staking (e.g., Ref Finance).
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum ReinvestOption {
    Burrow,
    Stake,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Preference {
    pub seed_id: String,
    pub token_id: AccountId,             
    pub smart_contract_name: AccountId,   
    pub is_active: bool,
    pub reinvest_to: ReinvestOption,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct User {
    pub wallet_id: AccountId,             
    pub subaccount_id: AccountId,       
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
    /// - subaccount_id: The AccountId representing the user’s subaccount ID.
    /// This function:
    /// - Uses the caller’s account ID (predecessor_account_id) as the wallet_id,
    /// - Creates a new User struct with the provided username, wallet_id, subaccount_id, and an empty preferences vector,
    /// - Inserts the user into the users map,
    /// - Logs the storage action,
    /// - Returns nothing (implicitly updates the contract state).
    pub fn store_user(&mut self, subaccount_id: AccountId) {
        let wallet_id = env::predecessor_account_id();
        let new_user = User {
            wallet_id: wallet_id.clone(), 
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
        assert_eq!(user.wallet_id, signer, "Unauthorized: wallet mismatch");
        user.preferences.extend(prefs);
        self.users.insert(&signer, &user);
        log!("Updated preferences for user: {}", signer);
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
    /// - Stores a user with username "alice" and subaccount accounts(1),
    /// - Retrieves the user by wallet_id (accounts(0)),
    /// - Asserts that the retrieved user’s username, wallet_id, and subaccount_id match the expected values.
    #[test]
    fn test_store_and_get_user() {
        let context = VMContextBuilder::new()
            .predecessor_account_id(accounts(0))
            .build();
        testing_env!(context);
        let mut contract = Contract::default();
        contract.store_user(accounts(1));
        let user = contract.get_user(accounts(0)).unwrap();
        assert_eq!(user.wallet_id, accounts(0));
        assert_eq!(user.subaccount_id, accounts(1));
    }

    /// A test to verify that updating a user’s preferences works correctly.
    /// This test:
    /// - Sets up a VM context with accounts(0) as the predecessor,
    /// - Initializes a default contract,
    /// - Stores a user with username "bob" and subaccount accounts(1),
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
        contract.store_user( accounts(1));
        let prefs = vec![Preference {
            seed_id: "seed1".to_string(),
            token_id: accounts(2),
            smart_contract_name: accounts(3),
            is_active: true,
            reinvest_to: ReinvestOption::Burrow,
        }];
        contract.update_preferences(prefs.clone());
        let user = contract.get_user(accounts(0)).unwrap();
        assert_eq!(user.preferences.len(), 1);
        assert_eq!(user.preferences[0].seed_id, prefs[0].seed_id);
        assert_eq!(user.preferences[0].is_active, true);
        assert!(matches!(user.preferences[0].reinvest_to, ReinvestOption::Burrow));
    }

   
}