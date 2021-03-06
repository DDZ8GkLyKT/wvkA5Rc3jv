use crate::crypto::hash::{Hashable, H256};
use crate::experiment::performance_counter::PayloadSize;
use bincode::serialize;
use ed25519_dalek::PublicKey;
use ed25519_dalek::Signature;
use std::cell::RefCell;
use std::hash::{Hash, Hasher};

use crate::config::*;
use std::sync::atomic::{AtomicU32, Ordering};

/// A unique identifier of a transaction output, a.k.a. a coin.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CoinId {
    /// The hash of the transaction that produces this coin.
    pub hash: H256,
    /// The index of the coin in the output list of the transaction that produces this coin.
    pub index: u32,
}

/// An address of a user. It is the SHA256 hash of the user's public key.
pub type Address = H256;

/// An input of a transaction.
//RR_comment: Need to add coinid / coin_shardid field and implementation here
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Input {
    /// The identifier of the input coin.
    pub coin: CoinId,
    /// The amount of this input.
    // TODO: this is redundant, since it is also stored in the transaction output. We need it to do
    // rollback.
    pub value: u64,
    /// The address of the owner of this input coin.
    // TODO: this is redundant, since it is also stored in the transaction output. We need it to do
    // rollback.
    pub owner: Address,
}

/// An output of a transaction.
// TODO: coinbase output (transaction fee). Maybe we don't need that in this case.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Output {
    /// The amount of this output.
    pub value: u64,
    /// The address of the recipient of this output coin.
    pub recipient: Address,
}

/// A Prism transaction. It takes a set of existing coins (inputs) and transforms them into a set
/// of coins (outputs).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    /// The list of inputs put into this transaction.
    pub input: Vec<Input>,
    /// The list of outputs generated by this transaction.
    pub output: Vec<Output>,
    /// Authorization of this transaction by the owners of the inputs.
    pub authorization: Vec<Authorization>,
    /// For intrashard we have shard_id as a part of transaction and not (input/output)
    pub shard_id: u32,
    #[serde(skip)]
    pub hash: RefCell<Option<H256>>,
}

impl PayloadSize for Transaction {
    /// Return the size in bytes
    fn size(&self) -> usize {
        return self.input.len() * std::mem::size_of::<Input>()
             + self.output.len() * std::mem::size_of::<Output>()
             + self.authorization.len() * std::mem::size_of::<Authorization>()
             + std::mem::size_of::<u32>();
    }
}

impl Hashable for Transaction {
    fn hash(&self) -> H256 {
        let hash = self.hash.borrow();
        if let Some(h) = *hash {
            return h;
        }
        drop(hash);
        let mut hash_mut = self.hash.borrow_mut();
        let hash: H256 =
            ring::digest::digest(&ring::digest::SHA256, &serialize(self).unwrap()).into();
        *hash_mut = Some(hash);
        return hash;
    }
}

/// Authorization of the transaction by the owner of an input coin.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Authorization {
    /// The public key of the owner.
    pub pubkey: Vec<u8>,
    /// The signature of the transaction input and output
    pub signature: Vec<u8>,
}

#[cfg(any(test, feature = "test-utilities"))]
pub mod tests {
    use super::*;
    use crate::crypto::hash::tests::generate_random_hash;
    use rand::{Rng, RngCore};

    pub fn generate_random_coinid() -> CoinId {
        let mut rng = rand::thread_rng();
        CoinId {
            hash: generate_random_hash(),
            index: rng.next_u32(),
        }
    }

    pub fn generate_random_input() -> Input {
        let mut rng = rand::thread_rng();
        Input {
            coin: generate_random_coinid(),
            value: rng.gen_range(1, 100),
            owner: generate_random_hash(),
        }
    }

    pub fn generate_random_output() -> Output {
        let mut rng = rand::thread_rng();
        Output {
            value: rng.gen_range(1, 100),
            recipient: generate_random_hash(),
        }
    }

    /*
    pub fn generate_random_transaction() -> Transaction {
        let mut rng = rand::thread_rng();
        let unsigned = Transaction {
            input: (0..rng.gen_range(1,5)).map(|_|generate_random_input()).collect(),
            output: (0..rng.gen_range(1,5)).map(|_|generate_random_output()).collect(),
            authorization: vec![],
            shard_id: SHARD_ID.load(Ordering::Relaxed),
            hash: RefCell::new(None),
        };
        let mut authorization = vec![];
        let keypair = KeyPair::random();
        authorization.push(Authorization {
            pubkey: keypair.public_key(),
            signature: unsigned.sign(&keypair),
        });
        Transaction {
            authorization,
            ..unsigned
        }
    }
    */
}
