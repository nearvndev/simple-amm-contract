use crate::*;

#[near_bindgen]
impl SimplePool {
    pub(crate) fn internal_transfer_share(&mut self, account_id: AccountId, receiver_id: AccountId, amount: u128) {
        let v_account = self.accounts.get(&account_id).unwrap();
        let mut account = Account::from(v_account);

        let v_receiver = self.accounts.get(&receiver_id).unwrap();
        let mut receiver = Account::from(v_receiver);

        assert!(account.share >= amount, "ERR_SHARE_AMOUNT_NOT_ENOUGH");

        account.share -= amount;
        receiver.share += amount;

        self.accounts.insert(&account_id, &VersionedAccount::from(account));
        self.accounts.insert(&receiver_id, &VersionedAccount::from(receiver));
    }

    /**
     * x = in_balance
     * y = out_balance
     * dx = amount_in
     * dy = amount_out
     * x*y = k => dy = y*dx / (x + dx)
     */
    pub(crate) fn internal_get_return(&self, token_in: AccountId, amount_in: Balance, token_out: AccountId) -> Balance {
        assert!(token_in != token_out, "ERR_NOT_GET_SAME_TOKEN");
        assert!(amount_in > 0, "ERR_AMOUNT_IN_INVALID");

        let in_balance = self.token_reserves.get(&token_in).unwrap();
        let out_balance = self.token_reserves.get(&token_out).unwrap();

        assert!(in_balance > 0 && out_balance > 0, "ERR_TOKEN_RESERVE_INVALID");

        let amount_out = U256::from(out_balance) * U256::from(amount_in) / U256::from(in_balance + amount_in);
        let amount_out_with_fee = U256::from(FEE_DIVISOR as u128 - self.exchange_fee) * amount_out / U256::from(FEE_DIVISOR);
        amount_out_with_fee.as_u128()
    }
}