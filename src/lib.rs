#![no_std]
#![feature(core_intrinsics, alloc_error_handler)]
//#![deny(warnings)]

extern crate alloc;

use borsh::{BorshDeserialize, BorshSerialize};
use near_sdk_pure::collections::UnorderedMap;
use near_sdk_pure::collections::UnorderedSet;
use near_sdk_pure::{env, near_bindgen, AccountId};
use alloc::{string::{String, ToString}, vec::Vec};

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[panic_handler]
#[no_mangle]
pub fn panic(info: &::core::panic::PanicInfo) -> ! {
    env::panic(info.to_string().as_bytes())
}

#[alloc_error_handler]
#[no_mangle]
pub fn oom(_: ::core::alloc::Layout) -> ! {
    ::core::intrinsics::abort();
}


/// This trait provides the baseline of functions as described at:
/// https://github.com/nearprotocol/NEPs/blob/nep-4/specs/Standards/Tokens/NonFungibleToken.md
pub trait NEP4 {
    // Grant the access to the given `accountId` for the given `tokenId`.
    // Requirements:
    // * The caller of the function (`predecessor_id`) should have access to the token.
    fn grant_access(&mut self, escrow_account_id: AccountId);

    // Revoke the access to the given `accountId` for the given `tokenId`.
    // Requirements:
    // * The caller of the function (`predecessor_id`) should have access to the token.
    fn revoke_access(&mut self, escrow_account_id: AccountId);

    // Transfer the given `tokenId` to the given `accountId`. Account `accountId` becomes the new owner.
    // Requirements:
    // * The caller of the function (`predecessor_id`) should have access to the token.
    fn transfer_from(&mut self, owner_id: AccountId, new_owner_id: AccountId, token_id: TokenId); 

    // Transfer the given `tokenId` to the given `accountId`. Account `accountId` becomes the new owner.
    // Requirements:
    // * The caller of the function (`predecessor_id`) should be the owner of the token. Callers who have
    // escrow access should use transfer_from.
    fn transfer(&mut self, new_owner_id: AccountId, token_id: TokenId); 

    // Returns `true` or `false` based on caller of the function (`predecessor_id) having access to the token
    fn check_access(&self, account_id: AccountId) -> bool;

    // Get an individual owner by given `tokenId`.
    fn get_token_owner(&self, token_id: TokenId) -> String;
}

/// The token ID type is also defined in the NEP
pub type TokenId = u64;
pub type AccountIdHash = Vec<u8>;

// Begin implementation
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct NonFungibleTokenBasic {
    pub token_to_account: UnorderedMap<TokenId, AccountId>,
    pub account_gives_access: UnorderedMap<AccountIdHash, UnorderedSet<AccountIdHash>>, // Vec<u8> is sha256 of account, makes it safer and is how fungible token also works
    pub owner_id: AccountId,
}

impl Default for NonFungibleTokenBasic {
    fn default() -> Self {
        panic!("NFT should be initialized before usage")
    }
}

#[near_bindgen]
impl NonFungibleTokenBasic {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        assert!(env::is_valid_account_id(owner_id.as_bytes()), "Owner's account ID is invalid.");
        assert!(!env::state_exists(), "Already initialized");
        Self {
            token_to_account: UnorderedMap::new(b"token-belongs-to".to_vec()),
            account_gives_access: UnorderedMap::new(b"gives-access".to_vec()),
            owner_id,
        }
    }
}

#[near_bindgen]
impl NEP4 for NonFungibleTokenBasic {
    fn grant_access(&mut self, escrow_account_id: AccountId) {
        let escrow_hash = env::sha256(escrow_account_id.as_bytes());
        let predecessor = env::predecessor_account_id();
        let predecessor_hash = env::sha256(predecessor.as_bytes());

        let mut access_set = match self.account_gives_access.get(&predecessor_hash) {
            Some(existing_set) => {
                existing_set
            },
            None => {
                UnorderedSet::new(b"new-access-set".to_vec())
            }
        };
        access_set.insert(&escrow_hash);
        self.account_gives_access.insert(&predecessor_hash, &access_set);
    }

    fn revoke_access(&mut self, escrow_account_id: AccountId) {
        let predecessor = env::predecessor_account_id();
        let predecessor_hash = env::sha256(predecessor.as_bytes());
        let mut existing_set = match self.account_gives_access.get(&predecessor_hash) {
            Some(existing_set) => existing_set,
            None => env::panic(b"Access does not exist.")
        };
        let escrow_hash = env::sha256(escrow_account_id.as_bytes());
        if existing_set.contains(&escrow_hash) {
            existing_set.remove(&escrow_hash);
            self.account_gives_access.insert(&predecessor_hash, &existing_set);
            env::log(b"Successfully removed access.")
        } else {
            env::panic(b"Did not find access for escrow ID.")
        }
    }

    fn transfer(&mut self, new_owner_id: AccountId, token_id: TokenId) {
        let token_owner_account_id = self.get_token_owner(token_id);
        let predecessor = env::predecessor_account_id();
        if predecessor != token_owner_account_id {
            env::panic(b"Attempt to call transfer on tokens belonging to another account.")
        }
        self.token_to_account.insert(&token_id, &new_owner_id);
    }

    fn transfer_from(&mut self, owner_id: AccountId, new_owner_id: AccountId, token_id: TokenId) {
        let token_owner_account_id = self.get_token_owner(token_id);
        if owner_id != token_owner_account_id {
            env::panic(b"Attempt to transfer a token from a different owner.")
        }

        if !self.check_access(token_owner_account_id) {
            env::panic(b"Attempt to transfer a token with no access.")
        }
        self.token_to_account.insert(&token_id, &new_owner_id);
    }

    fn check_access(&self, account_id: AccountId) -> bool {
        let account_hash = env::sha256(account_id.as_bytes());
        let predecessor = env::predecessor_account_id();
        if predecessor == account_id {
            return true;
        }
        match self.account_gives_access.get(&account_hash) {
            Some(access) => {
                let predecessor = env::predecessor_account_id();
                let predecessor_hash = env::sha256(predecessor.as_bytes());
                access.contains(&predecessor_hash)
            },
            None => false
        }
    }

    fn get_token_owner(&self, token_id: TokenId) -> String {
        match self.token_to_account.get(&token_id) {
            Some(owner_id) => owner_id,
            None => env::panic(b"No owner of the token ID specified")
        }
    }
}

/// Methods not in the strict scope of the NFT spec (NEP4)
#[near_bindgen]
impl NonFungibleTokenBasic {
    /// Creates a token for owner_id, doesn't use autoincrement, fails if id is taken
    pub fn mint_token(&mut self, owner_id: String, token_id: TokenId) {
        // make sure that only the owner can call this funtion
        self.only_owner();
        // Since Map doesn't have `contains` we use match
        let token_check = self.token_to_account.get(&token_id);
        if token_check.is_some() {
            env::panic(b"Token ID already exists.")
        }
        // No token with that ID exists, mint and add token to data structures
        self.token_to_account.insert(&token_id, &owner_id);
    }

    /// helper function determining contract ownership
    fn only_owner(&mut self) {
        assert_eq!(env::predecessor_account_id(), self.owner_id, "Only contract owner can call this method.");
    }
}

