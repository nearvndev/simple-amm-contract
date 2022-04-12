use std::collections::HashMap;

use near_sdk::collections::UnorderedMap;

use crate::*;


#[derive(BorshDeserialize, BorshSerialize)]
pub struct Account {
    pub near_amount: Balance,
    pub tokens: UnorderedMap<AccountId, Balance>,
    pub share: Balance
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AccountJson {
    pub tokens: HashMap<AccountId, Balance>,
    pub share: U128,
    pub account_id: AccountId
}

#[derive(BorshDeserialize, BorshSerialize)]
pub enum VersionedAccount {
    Current(Account)
}

impl From<VersionedAccount> for Account {
    fn from(v_account: VersionedAccount) -> Self {
        match v_account {
            VersionedAccount::Current(account) => account
        }
    }
}

impl From<Account> for VersionedAccount {
    fn from(account: Account) -> Self {
        VersionedAccount::Current(account)
    }
}

#[near_bindgen]
impl SimplePool {

    #[payable]
    pub fn register_account(&mut self, account_id: Option<AccountId>) {
        assert_at_least_one_yocto();
        let account_id_unwrap = account_id.unwrap_or_else(|| env::predecessor_account_id());
        let v_account = self.accounts.get(&account_id_unwrap);
        assert!(v_account.is_none(), "ERR_ACCOUNT_REGISTED");

        let before_storage_usage = env::storage_usage();

        let account = Account {
            near_amount: 0,
            tokens: UnorderedMap::new(StorageKey::AccountTokenKey { account_id: account_id_unwrap.clone() }),
            share: 0
        };

        self.accounts.insert(&account_id_unwrap, &VersionedAccount::from(account));

        let after_storage_usage = env::storage_usage();

        refund_deposit(after_storage_usage - before_storage_usage);
    }

    pub fn get_account_info(&self, account_id: AccountId) -> AccountJson {
        let v_account = self.accounts.get(&account_id).unwrap();
        let account = Account::from(v_account);

        AccountJson { tokens: HashMap::from_iter(account.tokens.to_vec()), share: U128(account.share), account_id }
    }

    pub fn storage_balance_of(&self, account_id: AccountId) -> U128 {
        let v_account = self.accounts.get(&account_id);

        if v_account.is_some() {
            U128(1)
        } else {
            U128(0)
        }
    }

    pub(crate) fn internal_deposit(&mut self, account_id: AccountId, token_id: AccountId, amount: u128) {
        let v_account = self.accounts.get(&account_id).unwrap();
        let mut account = Account::from(v_account);

        let token_amount = account.tokens.get(&token_id).unwrap_or_else( || 0);
        account.tokens.insert(&token_id, &(token_amount + amount));

        self.accounts.insert(&account_id, &VersionedAccount::from(account));
    }

    pub(crate) fn internal_withdraw(&mut self, account_id: AccountId, token_id: AccountId, amount: u128) {
        let v_account = self.accounts.get(&account_id).unwrap();
        let mut account = Account::from(v_account);

        let token_amount = account.tokens.get(&token_id).unwrap_or_else( || 0);

        assert!(token_amount >= amount, "ERR_INVALID_AMOUNT");
        
        account.tokens.insert(&token_id, &(token_amount - amount));

        self.accounts.insert(&account_id, &VersionedAccount::from(account));
    }
}