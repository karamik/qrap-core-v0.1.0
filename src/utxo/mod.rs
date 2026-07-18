
# src/utxo/mod.rs - UTXO engine + Epoch Nullifier Tree
utxo_mod = '''//! UTXO Engine + Epoch Nullifier Tree
//! 
//! Ядро системы: отслеживание непотраченных выходов и предотвращение double-spending

use crate::types::*;
use crate::crypto::*;
use std::collections::{HashMap, HashSet};
use tracing::{info, warn, debug};

/// UTXO Engine — хранит все непотраченные выходы
pub struct UTXOEngine {
    /// Текущая эпоха
    current_epoch: u32,
    
    /// Текущий номер блока в эпохе
    current_block: u64,
    
    /// UTXO: coin_id -> UTXO
    utxos: HashMap<u64, UTXO>,
    
    /// Следующий свободный coin_id
    next_coin_id: u64,
    
    /// Текущее Merkle root UTXO
    utxo_root: Hash,
    
    /// Текущее Nullifier SMT (упрощённо: HashSet)
    current_nullifiers: HashSet<Nullifier>,
    
    /// Корни архивных эпох
    archived_nullifier_roots: Vec<Hash>,
    
    /// История блоков
    blocks: Vec<Block>,
}

#[derive(Debug, thiserror::Error)]
pub enum UTXOError {
    #[error("Double spend: nullifier already exists")]
    DoubleSpend,
    
    #[error("Invalid epoch: coin expired or from future")]
    InvalidEpoch,
    
    #[error("Invalid proof")]
    InvalidProof,
    
    #[error("Value conservation violated")]
    ConservationViolation,
    
    #[error("UTXO not found: {0}")]
    UTXONotFound(u64),
}

impl UTXOEngine {
    pub fn new() -> Self {
        let genesis = Block::genesis();
        Self {
            current_epoch: 0,
            current_block: 0,
            utxos: HashMap::new(),
            next_coin_id: 1,
            utxo_root: Hash([0u8; 32]),
            current_nullifiers: HashSet::new(),
            archived_nullifier_roots: vec![Hash([0u8; 32])],
            blocks: vec![genesis],
        }
    }
    
    /// Создать новый UTXO (mint)
    pub fn mint(&mut self, owner: Address, amount: u64, secret: &[u8]) -> Result<UTXO, UTXOError> {
        let coin_id = self.next_coin_id;
        self.next_coin_id += 1;
        
        // Генерируем commitment: C = hash(s, e, v)
        // Пока упрощённо: secret = s || e
        let commitment = Commitment::new(secret, &[0u8; 32], amount);
        
        let utxo = UTXO {
            commitment,
            coin_id,
            epoch_created: self.current_epoch,
            owner,
        };
        
        self.utxos.insert(coin_id, utxo.clone());
        self.update_utxo_root();
        
        debug!("Minted UTXO coin_id={} epoch={}", coin_id, self.current_epoch);
        Ok(utxo)
    }
    
    /// Потратить UTXO (spend)
    pub fn spend(
        &mut self,
        tx: &Transaction,
    ) -> Result<Vec<UTXO>, UTXOError> {
        // 1. Проверка epoch window
        for input in &tx.inputs {
            let age = self.current_epoch.saturating_sub(input.epoch_created);
            if age > 30 {  // Spend window = 30 epochs
                return Err(UTXOError::InvalidEpoch);
            }
        }
        
        // 2. Проверка double-spend
        if self.current_nullifiers.contains(&tx.nullifier) {
            return Err(UTXOError::DoubleSpend);
        }
        
        // 3. Проверка conservation of value (упрощённо)
        let total_input: u64 = tx.inputs.iter().map(|u| u.commitment.amount).sum();
        let total_output: u64 = tx.outputs.iter().map(|c| c.amount).sum();
        if total_input != total_output + tx.fee {
            return Err(UTXOError::ConservationViolation);
        }
        
        // 4. Добавить nullifier
        self.current_nullifiers.insert(tx.nullifier.clone());
        
        // 5. Удалить spent inputs
        for input in &tx.inputs {
            self.utxos.remove(&input.coin_id);
        }
        
        // 6. Добавить outputs
        let mut new_utxos = Vec::new();
        for (i, output) in tx.outputs.iter().enumerate() {
            let coin_id = self.next_coin_id + i as u64;
            let utxo = UTXO {
                commitment: output.clone(),
                coin_id,
                epoch_created: self.current_epoch,
                owner: tx.inputs[0].owner.clone(),  // Simplified
            };
            self.utxos.insert(coin_id, utxo.clone());
            new_utxos.push(utxo);
        }
        self.next_coin_id += tx.outputs.len() as u64;
        
        self.update_utxo_root();
        
        debug!("Spent nullifier={:?}, created {} new UTXOs", tx.nullifier, new_utxos.len());
        Ok(new_utxos)
    }
    
    /// Создать блок из pending transactions
    pub fn create_block(&mut self, transactions: Vec<Transaction>, proposer: Address) -> Block {
        self.current_block += 1;
        
        // Check epoch rollover
        if self.current_block % crate::EPOCH_SIZE == 0 {
            self.rollover_epoch();
        }
        
        let prev_hash = self.blocks.last()
            .map(|b| hash(&bincode::serialize(b).unwrap_or_default()))
            .unwrap_or_else(|| Hash([0u8; 32]));
        
        let nullifier_root = self.compute_nullifier_root();
        
        let block = Block {
            height: self.current_block,
            epoch: self.current_epoch,
            prev_hash,
            utxo_root: self.utxo_root.clone(),
            nullifier_root,
            transactions,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            proposer,
        };
        
        self.blocks.push(block.clone());
        info!("Created block height={} epoch={} txs={}", 
              block.height, block.epoch, block.transactions.len());
        
        block
    }
    
    /// Rollover epoch: freeze nullifier set, start new
    fn rollover_epoch(&mut self) {
        let old_root = self.compute_nullifier_root();
        self.archived_nullifier_roots.push(old_root);
        
        // Clear current nullifiers (they're now in archive)
        // In production: persist to DA layer
        self.current_nullifiers.clear();
        
        self.current_epoch += 1;
        info!("Epoch rollover: {} -> {}", self.current_epoch - 1, self.current_epoch);
    }
    
    /// Вычислить Merkle root всех UTXO
    fn update_utxo_root(&mut self) {
        let leaves: Vec<Hash> = self.utxos.values()
            .map(|u| u.commitment.value.clone())
            .collect();
        self.utxo_root = merkle_root(&leaves);
    }
    
    /// Вычислить root текущего nullifier set
    fn compute_nullifier_root(&self) -> Hash {
        let mut nullifiers: Vec<_> = self.current_nullifiers.iter().collect();
        nullifiers.sort_by(|a, b| a.0.cmp(&b.0));
        
        let leaves: Vec<Hash> = nullifiers.iter()
            .map(|n| Hash(n.0))
            .collect();
        
        merkle_root(&leaves)
    }
    
    /// Получить UTXO по coin_id
    pub fn get_utxo(&self, coin_id: u64) -> Option<&UTXO> {
        self.utxos.get(&coin_id)
    }
    
    /// Проверить, потрачен ли nullifier
    pub fn is_spent(&self, nullifier: &Nullifier) -> bool {
        self.current_nullifiers.contains(nullifier)
    }
    
    /// Статистика
    pub fn stats(&self) -> EngineStats {
        EngineStats {
            epoch: self.current_epoch,
            block: self.current_block,
            utxo_count: self.utxos.len(),
            nullifier_count: self.current_nullifiers.len(),
            archived_epochs: self.archived_nullifier_roots.len() as u32,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EngineStats {
    pub epoch: u32,
    pub block: u64,
    pub utxo_count: usize,
    pub nullifier_count: usize,
    pub archived_epochs: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mint_and_spend() {
        let mut engine = UTXOEngine::new();
        let owner = Address([1u8; 32]);
        let secret = b"test_secret";
        
        // Mint
        let utxo1 = engine.mint(owner.clone(), 1000, secret).unwrap();
        assert_eq!(engine.utxos.len(), 1);
        
        // Create transaction
        let nullifier = Nullifier::derive(secret, utxo1.coin_id, utxo1.epoch_created);
        let output = Commitment::new(b"new_secret", &[0u8; 32], 900);
        
        let tx = Transaction {
            nullifier,
            inputs: vec![utxo1],
            outputs: vec![output],
            fee: 100,
            proof: vec![],  // ZK-proof placeholder
        };
        
        // Spend
        let new_utxos = engine.spend(&tx).unwrap();
        assert_eq!(new_utxos.len(), 1);
        assert_eq!(engine.utxos.len(), 1);  // 1 spent, 1 created
        
        // Double spend should fail
        assert!(engine.spend(&tx).is_err());
    }
    
    #[test]
    fn test_epoch_rollover() {
        let mut engine = UTXOEngine::new();
        let owner = Address([1u8; 32]);
        
        // Create blocks to trigger rollover
        for _ in 0..crate::EPOCH_SIZE + 1 {
            engine.create_block(vec![], owner.clone());
        }
        
        assert_eq!(engine.current_epoch, 1);
        assert_eq!(engine.archived_nullifier_roots.len(), 2);  // genesis + epoch 0
    }
}
'''

with open(os.path.join(base_dir, "src/utxo/mod.rs"), "w") as f:
    f.write(utxo_mod)

print("src/utxo/mod.rs created")
