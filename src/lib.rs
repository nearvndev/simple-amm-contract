use near_sdk::{env, near_bindgen, AccountId, Balance, BorshStorageKey, PanicOnDefault, Promise, log, ext_contract, Gas, PromiseResult};
use near_sdk::collections::{LookupMap, Vector, UnorderedMap};
use near_sdk::json_types::{U128};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Serialize, Deserialize};
use uint::construct_uint;

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}

use crate::util::*;
use crate::account::*;
mod share;
mod util;
mod internal;
mod core_impl;
mod account;
mod ft_receiver;

/// Fee divisor, allowing to provide fee in bps.
pub const FEE_DIVISOR: u32 = 10_000;

/// Initial shares supply on deposit of liquidity.
pub const INIT_SHARES_SUPPLY: u128 = 1_000_000_000_000_000_000_000_000;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SwapVolume {
    pub input: u128,
    pub output: u128,
}

impl Default for SwapVolume {
    fn default() -> Self {
        Self {
            input: 0,
            output: 0,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    TokenIDKey,
    TokenReserveKey,
    ShareKey,
    VolumeKey,
    AccountKey,
    AccountTokenKey {
        account_id: AccountId
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct SimplePool {
    pub owner_id: AccountId,
    pub token_ids: Vector<AccountId>,
    pub token_reserves: LookupMap<AccountId, Balance>,
    pub total_share_supply: Balance,
    pub exchange_fee: Balance, // ex fee 0.3% => 300/FEE_DIVISOR => exchange_fee = 300
    pub volumes: UnorderedMap<AccountId, SwapVolume>,
    pub accounts: LookupMap<AccountId, VersionedAccount>
}

#[near_bindgen]
impl SimplePool {
    #[init]
    pub fn new(owner_id: AccountId, token_ids: Vec<AccountId>, exchange_fee: U128) -> Self {
        assert_eq!(token_ids.len(), 2, "MUST 2 TOKEN IDS");
        let mut tokens = Vector::new(StorageKey::TokenIDKey);
        tokens.push(&token_ids[0]);
        tokens.push(&token_ids[1]);

        let mut token_reserves = LookupMap::new(StorageKey::TokenReserveKey);
        token_reserves.insert(&token_ids[0], &0);
        token_reserves.insert(&token_ids[1], &0);

        let mut volumes = UnorderedMap::new(StorageKey::VolumeKey);
        volumes.insert(&token_ids[0], &SwapVolume::default());
        volumes.insert(&token_ids[1], &SwapVolume::default());

        SimplePool { 
            owner_id,
            token_ids: tokens, 
            token_reserves,
            total_share_supply: 0, 
            exchange_fee: exchange_fee.0,
            volumes,
            accounts: LookupMap::new(StorageKey::AccountKey)
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
