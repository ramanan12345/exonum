use exonum::storage::{Snapshot, ProofMapIndex, ProofListIndex, Fork};
use exonum::crypto::{Hash, PublicKey};
use exonum::blockchain::gen_prefix;

use std::fmt;

use wallet::Wallet;
use INITIAL_BALANCE;

/// Database schema for the cryptocurrency.
pub struct CurrencySchema<T> {
    view: T,
}

impl<T> AsMut<T> for CurrencySchema<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.view
    }
}

impl<T> fmt::Debug for CurrencySchema<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CurrencySchema {{}}")
    }
}

impl<T> CurrencySchema<T>
where
    T: AsRef<Snapshot>,
{
    /// Constructs schema from the database view.
    pub fn new(view: T) -> Self {
        CurrencySchema { view }
    }

    /// Returns `MerklePatriciaTable` with wallets.
    pub fn wallets(&self) -> ProofMapIndex<&T, PublicKey, Wallet> {
        ProofMapIndex::new("cryptocurrency.wallets", &self.view)
    }

    /// Returns history of the wallet with the given public key.
    pub fn wallet_history(&self, public_key: &PublicKey) -> ProofListIndex<&T, Hash> {
        ProofListIndex::with_prefix(
            "cryptocurrency.wallet_history",
            gen_prefix(public_key),
            &self.view,
        )
    }

    /// Returns wallet for the given public key.
    pub fn wallet(&self, pub_key: &PublicKey) -> Option<Wallet> {
        self.wallets().get(pub_key)
    }

    /// Returns database state hash.
    pub fn state_hash(&self) -> Vec<Hash> {
        vec![self.wallets().root_hash()]
    }
}

/// Implementation of mutable methods.
impl<'a> CurrencySchema<&'a mut Fork> {
    /// Returns mutable `MerklePatriciaTable` with wallets.
    pub fn wallets_mut(&mut self) -> ProofMapIndex<&mut Fork, PublicKey, Wallet> {
        ProofMapIndex::new("cryptocurrency.wallets", &mut self.view)
    }

    /// Returns history for the wallet by the given public key.
    pub fn wallet_history_mut(
        &mut self,
        public_key: &PublicKey,
    ) -> ProofListIndex<&mut Fork, Hash> {
        ProofListIndex::with_prefix(
            "cryptocurrency.wallet_history",
            gen_prefix(public_key),
            &mut self.view,
        )
    }

    /// Update balance of the wallet and append new record to its history.
    ///
    /// Panics if there is no wallet with given public key.
    pub fn set_wallet_balance(&mut self, key: &PublicKey, balance: u64, transaction: &Hash) {
        let wallet = {
            let wallet = self.wallet(key).unwrap();
            let mut history = self.wallet_history_mut(key);
            history.push(*transaction);
            let history_hash = history.root_hash();
            wallet.set_balance(balance, &history_hash)
        };
        self.wallets_mut().put(key, wallet);
    }

    /// Create new wallet and append first record to its history.
    pub fn create_wallet(&mut self, key: &PublicKey, name: &str, transaction: &Hash) {
        let wallet = {
            let mut history = self.wallet_history_mut(key);
            history.push(*transaction);
            let history_hash = history.root_hash();
            Wallet::new(key, name, INITIAL_BALANCE, history.len(), &history_hash)
        };
        self.wallets_mut().put(key, wallet);
    }
}
