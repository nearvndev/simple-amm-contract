use crate::*;

#[near_bindgen]
impl SimplePool {
    pub fn get_share_of(&self, account_id: AccountId) -> U128 {
        let v_account = self.accounts.get(&account_id).unwrap();
        let account = Account::from(v_account);

        U128(account.share)
    }

    #[payable]
    pub fn transfer_share(&mut self, receiver_id: AccountId, amount: U128) {
        assert_one_yocto();
        let account_id = env::predecessor_account_id();
        self.internal_transfer_share(account_id, receiver_id, amount.0);
    }

    pub fn get_total_share_balance(&self) -> U128 {
        U128(self.total_share_supply)
    }

    pub(crate) fn mint_share(&mut self, account_id: AccountId, amount: Balance) {
        let v_account = self.accounts.get(&account_id).unwrap();
        let mut account = Account::from(v_account);
        account.share += amount;

        self.total_share_supply += amount;

        self.accounts.insert(&account_id, &VersionedAccount::from(account));
    }

    pub(crate) fn burn_share(&mut self, account_id: AccountId, amount: Balance) {
        let v_account = self.accounts.get(&account_id).unwrap();
        let mut account = Account::from(v_account);

        assert!(account.share >= amount);
        account.share -= amount;
        self.total_share_supply -= amount;

        self.accounts.insert(&account_id, &VersionedAccount::from(account));
    }
}