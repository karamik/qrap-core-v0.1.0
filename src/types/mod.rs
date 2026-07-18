
# src/types/mod.rs - базовые типы
base_dir = "/mnt/agents/output/qrap-core"

types_mod = '''//! Базовые типы QRAP

use serde::{Deserialize, Serialize};
use std::fmt;

/// 32-байтный хэш
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash(pub [u8; 32]);

impl Hash {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
    
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
    
    /// Плейсхолдер для Poseidon-256
    pub fn poseidon(inputs: &[Hash]) -> Self {
        use sha3::{Digest, Sha3_256};
        let mut hasher = Sha3_256::new();
        for input in inputs {
            hasher.update(&input.0);
        }
        let result = hasher.finalize();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&result);
        Self(bytes)
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0[..8]))
    }
}

/// Адрес (стелс-адрес на базе ML-KEM, пока - хэш публичного ключа)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address(pub [u8; 32]);

impl Address {
    pub fn from_pubkey(pubkey: &[u8]) -> Self {
        use sha3::{Digest, Sha3_256};
        let hash = Sha3_256::digest(pubkey);
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&hash);
        Self(bytes)
    }
}

/// Ring-LWE Commitment: C = a·s + e + Encode(v)
/// Пока упрощённая версия: хэш (s, e, v)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Commitment {
    pub value: Hash,
    pub amount: u64,  // Для отладки, в production - скрыто
}

impl Commitment {
    pub fn new(s: &[u8], e: &[u8], v: u64) -> Self {
        let mut hasher_input = Vec::new();
        hasher_input.extend_from_slice(s);
        hasher_input.extend_from_slice(e);
        hasher_input.extend_from_slice(&v.to_le_bytes());
        
        use sha3::{Digest, Sha3_256};
        let hash = Sha3_256::digest(&hasher_input);
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&hash);
        
        Self {
            value: Hash(bytes),
            amount: v,  // TODO: убрать в production
        }
    }
}

/// Nullifier: N = Poseidon-256(s || coin_id || epoch)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Nullifier(pub [u8; 32]);

impl Nullifier {
    pub fn derive(s: &[u8], coin_id: u64, epoch: u32) -> Self {
        let mut hasher_input = Vec::new();
        hasher_input.extend_from_slice(s);
        hasher_input.extend_from_slice(&coin_id.to_le_bytes());
        hasher_input.extend_from_slice(&epoch.to_le_bytes());
        
        use sha3::{Digest, Sha3_256};
        let hash = Sha3_256::digest(&hasher_input);
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&hash);
        
        Self(bytes)
    }
}

/// UTXO - непотраченный выход
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTXO {
    pub commitment: Commitment,
    pub coin_id: u64,
    pub epoch_created: u32,
    pub owner: Address,
}

/// Транзакция (упрощённая)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub nullifier: Nullifier,
    pub inputs: Vec<UTXO>,
    pub outputs: Vec<Commitment>,
    pub fee: u64,
    pub proof: Vec<u8>,  // ZK-proof placeholder
}

/// Блок
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub height: u64,
    pub epoch: u32,
    pub prev_hash: Hash,
    pub utxo_root: Hash,
    pub nullifier_root: Hash,
    pub transactions: Vec<Transaction>,
    pub timestamp: u64,
    pub proposer: Address,
}

/// Genesis блок
impl Block {
    pub fn genesis() -> Self {
        Self {
            height: 0,
            epoch: 0,
            prev_hash: Hash([0u8; 32]),
            utxo_root: Hash([0u8; 32]),
            nullifier_root: Hash([0u8; 32]),
            transactions: vec![],
            timestamp: 1728000000,  // 2024-10-04 ~
            proposer: Address([0u8; 32]),
        }
    }
}

// hex encode helper
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}
'''

with open(os.path.join(base_dir, "src/types/mod.rs"), "w") as f:
    f.write(types_mod)

print("src/types/mod.rs created")
