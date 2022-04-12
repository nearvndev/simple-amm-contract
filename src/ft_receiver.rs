use near_sdk::PromiseOrValue;

use crate::*;

pub trait FungibleTokenReceiver {
    fn ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String) -> PromiseOrValue<U128>;
}

#[ext_contract(ext_ft_contract)]
pub trait FungibleTokenCore {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
}

#[near_bindgen]
impl FungibleTokenReceiver for SimplePool {
    /**
     * Handle deposit token
     */
    fn ft_on_transfer(&mut self, sender_id: AccountId, amount: U128, msg: String) -> PromiseOrValue<U128> {
        let contract_id = env::predecessor_account_id();
        assert!(contract_id == self.token_ids.get(0).unwrap() || contract_id == self.token_ids.get(1).unwrap(), "ERR_FT_CONTRACT_NOT_IN_POOL");
        self.internal_deposit(sender_id, contract_id, amount.0);

        PromiseOrValue::Value(U128(0))
    }
}