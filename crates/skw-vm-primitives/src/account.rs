use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

use crate::contract_runtime::{CryptoHash, Balance, StorageUsage};
use crate::serialize::{u128_dec_format_compatible};

// TODO: Nonce!!!

/// Per account information stored in the state.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct Account {
    /// The total not locked tokens.
    #[serde(with = "u128_dec_format_compatible")]
    amount: Balance,
    /// The amount locked due to staking.
    #[serde(with = "u128_dec_format_compatible")]
    locked: Balance,
    /// Hash of the code stored in the storage for this account.
    code_hash: CryptoHash,
    /// Storage used by the given account, includes account id, this struct, access keys and other data.
    storage_usage: StorageUsage,
}

impl Account {
    /// Max number of bytes an account can have in its state (excluding contract code)
    /// before it is infeasible to delete.
    pub const MAX_ACCOUNT_DELETION_STORAGE_USAGE: u64 = 10_000;

    pub fn new(
        amount: Balance,
        locked: Balance,
        code_hash: CryptoHash,
        storage_usage: StorageUsage,
    ) -> Self {
        Account { amount, locked, code_hash, storage_usage }
    }

    #[inline]
    pub fn amount(&self) -> Balance {
        self.amount
    }

    #[inline]
    pub fn locked(&self) -> Balance {
        self.locked
    }

    #[inline]
    pub fn code_hash(&self) -> CryptoHash {
        self.code_hash
    }

    #[inline]
    pub fn storage_usage(&self) -> StorageUsage {
        self.storage_usage
    }

    #[inline]
    pub fn set_amount(&mut self, amount: Balance) {
        self.amount = amount;
    }

    #[inline]
    pub fn set_locked(&mut self, locked: Balance) {
        self.locked = locked;
    }

    #[inline]
    pub fn set_code_hash(&mut self, code_hash: CryptoHash) {
        self.code_hash = code_hash;
    }

    #[inline]
    pub fn set_storage_usage(&mut self, storage_usage: StorageUsage) {
        self.storage_usage = storage_usage;
    }
}

#[cfg(test)]
mod tests {
    use borsh::BorshSerialize;

    use crate::contract_runtime::hash_bytes;
    use crate::serialize::to_base;

    use super::*;

    #[test]
    fn test_account_serialization() {
        let acc = Account::new(1_000_000, 1_000_000, CryptoHash::default(), 100);
        let bytes = acc.try_to_vec().unwrap();
        assert_eq!(to_base(&hash_bytes(&bytes)), "EVk5UaxBe8LQ8r8iD5EAxVBs6TJcMDKqyH7PBuho6bBJ");
    }

    #[test]
    fn test_account_deserialization() {
        let original_account = Account {
            amount: 100,
            locked: 200,
            code_hash: CryptoHash::default(),
            storage_usage: 300,
        };
        let mut original_bytes = &original_account.try_to_vec().unwrap()[..];
        let new_account = <Account as BorshDeserialize>::deserialize(&mut original_bytes).unwrap();
        assert_eq!(new_account.amount, original_account.amount);
        assert_eq!(new_account.locked, original_account.locked);
        assert_eq!(new_account.code_hash, original_account.code_hash);
        assert_eq!(new_account.storage_usage, original_account.storage_usage);
        
        let mut new_bytes = &new_account.try_to_vec().unwrap()[..];
        let deserialized_account =
            <Account as BorshDeserialize>::deserialize(&mut new_bytes).unwrap();
        assert_eq!(deserialized_account, new_account);
    }
}
