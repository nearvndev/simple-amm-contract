use near_sdk::PromiseOrValue;

use crate::*;
use std::{cmp::min, collections::HashMap, hash::Hash};

pub const FT_TRANSFER_GAS: Gas = 10_000_000_000_000;
pub const WITHDRAW_CALLBACK_GAS: Gas = 10_000_000_000_000;


#[ext_contract(ext_ft_contract)]
pub trait FungibleTokenCore {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
}

#[ext_contract(ext_self)]
pub trait ExtStakingContract {
    fn ft_withdraw_callback(&mut self, account_id: AccountId, token_id: AccountId, amount: U128);
}

#[near_bindgen]
impl SimplePool {
    pub fn get_tokens(&self) -> Vec<AccountId> {
        self.token_ids.to_vec()
    }

    pub fn get_fee(&self) -> U128 {
        U128(self.exchange_fee)
    }

    pub fn get_volumes(&self) -> Vec<SwapVolume> {
        self.volumes.values().into_iter().collect()
    }

    pub fn get_return(&self, token_in: AccountId, amount_in: U128, token_out: AccountId) -> U128 {
        U128(self.internal_get_return(token_in, amount_in.0, token_out))
    }

    #[payable]
    pub fn swap(&mut self, token_in: AccountId, amount_in: U128, token_out: AccountId, min_amount_out: U128) -> Balance {
        assert_one_yocto();
        let in_reserve = self.token_reserves.get(&token_in).unwrap();
        let out_reserve = self.token_reserves.get(&token_out).unwrap();

        let amount_out = self.internal_get_return(token_in.clone(), amount_in.0, token_out.clone());
        assert!(amount_out >= min_amount_out.0, "ERR_MIN_AMOUNT");
        log!("Swapped from {} {} for {} {} ", token_in.clone(), amount_in.clone().0.to_string(), token_out.clone(), amount_out.clone().to_string());

        self.token_reserves.insert(&token_in, &(in_reserve + amount_in.0));
        self.token_reserves.insert(&token_out, &(&out_reserve - amount_out));


        // Deposit amount out for user
        self.internal_deposit(env::predecessor_account_id(), token_out.clone(), amount_out.clone());

        // Wihdraw amount in
        self.internal_withdraw(env::predecessor_account_id(), token_in.clone(), amount_in.clone().0);

        // Add volume
        let mut volume = self.volumes.get(&token_in).unwrap_or_else( || SwapVolume::default());
        volume.input += amount_in.0;
        volume.output += amount_out;

        self.volumes.insert(&token_in, &volume);

        amount_out
    }

    /**
         * calculate share
         * How much dx, dy to add?
         * xy = k
         * (x + dx)(y + dy) = k'
         * 
         * No price change, before and after adding liquidity
         * x / y = (x + dx) / (y + dy)
         * x(y + dy) = y(x + dx)
         * x * dy = y * dx
         * x / y = dx / dy
         * dy = y / x * dx
    */

    #[payable]
    pub fn add_liquidity(&mut self, token_in: AccountId, amount_in: U128, token_out: AccountId, amount_out: U128) -> Balance {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();

        assert!(token_in != token_out, "ERR_TOKEN_ID");
        assert!(amount_in.0 > 0 && amount_out.0 > 0, "ERR_INVALID_AMOUNT");

        // Withdraw token
        self.internal_withdraw(account_id.clone(), token_in.clone(), amount_in.clone().0);
        self.internal_withdraw(account_id.clone(), token_out.clone(), amount_out.clone().0);

        let in_reserve = self.token_reserves.get(&token_in).unwrap();
        let out_reserve = self.token_reserves.get(&token_out).unwrap();

        let share = if self.total_share_supply > 0 {
            assert!(in_reserve > 0 && out_reserve > 0, "ERR_RESERVE_INVALID");
            assert_eq!(U256::from(in_reserve) * U256::from(amount_out.0), U256::from(out_reserve) * U256::from(amount_in.0), "x / y != dx / dy");

            self.token_reserves.insert(&token_in, &(in_reserve + amount_in.0));
            self.token_reserves.insert(&token_out, &(out_reserve + amount_out.0));

            min(
                U256::from(amount_in.clone().0) * U256::from(self.total_share_supply) / U256::from(in_reserve.clone()),  
                U256::from(amount_out.clone().0) * U256::from(self.total_share_supply) / U256::from(out_reserve.clone())
            )
        } else {
            self.token_reserves.insert(&token_in, &(in_reserve + amount_in.0));
            self.token_reserves.insert(&token_out, &(out_reserve + amount_out.0));
            U256::from(INIT_SHARES_SUPPLY)
        };

        assert!(share.as_u128() > 0, "Share = 0");

        self.mint_share(account_id, share.clone().as_u128());
        share.as_u128()
    }

    #[payable]
    pub fn remove_liquidity(&mut self, amount: U128) {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        // Check and burn share
        self.burn_share(account_id.clone(), amount.0);

        let token_1 = self.token_ids.get(0).unwrap();
        let token_2 = self.token_ids.get(1).unwrap();

        let reserve_1 = self.token_reserves.get(&token_1).unwrap();
        let reserve_2 = self.token_reserves.get(&token_2).unwrap();

        let amount_out_1 = U256::from(amount.0) * U256::from(reserve_1) / U256::from(self.total_share_supply);
        let amount_out_2 = U256::from(amount.0) * U256::from(reserve_2) / U256::from(self.total_share_supply);

        self.token_reserves.insert(&token_1, &(reserve_1 - amount_out_1.as_u128()));
        self.token_reserves.insert(&token_2, &(reserve_2 - amount_out_2.as_u128()));

        self.internal_deposit(account_id.clone(), token_1.clone(), amount_out_1.as_u128());
        self.internal_deposit(account_id.clone(), token_2.clone(), amount_out_2.as_u128());
    }

    #[payable]
    pub fn withdraw(&mut self, token_id: AccountId) -> Promise {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        let v_account = self.accounts.get(&account_id).unwrap();
        let account = Account::from(v_account);

        let token_balance = account.tokens.get(&token_id).unwrap();
        assert!(token_balance > 0, "ERR_TOKEN_BALANCE_EQUAL_ZERO");

        // Transfer all token balance

        ext_ft_contract::ft_transfer(
            account_id.clone(), 
            U128(token_balance.clone()), 
            Some(String::from("Withdraw from exchange")), 
            &token_id, 
            1, 
            FT_TRANSFER_GAS
        ).then(ext_self::ft_withdraw_callback(
            account_id, 
            token_id, 
            U128(token_balance), 
            &env::current_account_id(), 
            0, 
            WITHDRAW_CALLBACK_GAS
        ))
    }

    #[private]
    pub fn ft_withdraw_callback(&mut self, account_id: AccountId, token_id: AccountId, amount: U128) -> PromiseOrValue<U128> {
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_value) => {
                self.internal_withdraw(account_id, token_id, amount.0);
                PromiseOrValue::Value(amount)
            },
            PromiseResult::Failed => env::panic(b"ERR_CALL_FAILED"),
        }
    }
}