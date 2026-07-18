
# src/consensus/mod.rs - базовый BFT консенсус (упрощённый HotStuff)
consensus_mod = '''//! Базовый BFT консенсус (упрощённый HotStuff)
//! 
//! Phase: PROPOSE -> PREPARE -> COMMIT -> DECIDE
//! Упрощение: без View Change, без threshold signatures

use crate::types::*;
use crate::utxo::{UTXOEngine, UTXOError};
use std::collections::{HashMap, HashSet};
use tracing::{info, warn, debug};

/// Валидатор
#[derive(Debug, Clone)]
pub struct Validator {
    pub address: Address,
    pub stake: u64,
    pub name: String,
}

/// Состояние консенсуса
pub struct ConsensusEngine {
    /// Наш адрес
    my_address: Address,
    
    /// Список валидаторов
    validators: Vec<Validator>,
    
    /// UTXO engine
    utxo: UTXOEngine,
    
    /// Текущая view (round)
    view: u64,
    
    /// Высота последнего решённого блока
    decided_height: u64,
    
    /// Prepare certificates: height -> set of validators
    prepare_certs: HashMap<u64, HashSet<Address>>,
    
    /// Commit certificates: height -> set of validators
    commit_certs: HashMap<u64, HashSet<Address>>,
    
    /// Pending transactions
    pending_txs: Vec<Transaction>,
    
    /// Решённые блоки
    decided_blocks: Vec<Block>,
}

#[derive(Debug, thiserror::Error)]
pub enum ConsensusError {
    #[error("Not a validator")]
    NotValidator,
    
    #[error("Invalid proposal")]
    InvalidProposal,
    
    #[error("Quorum not reached")]
    QuorumNotReached,
    
    #[error("UTXO error: {0}")]
    UTXO(#[from] UTXOError),
    
    #[error("Double vote")]
    DoubleVote,
}

impl ConsensusEngine {
    pub fn new(my_address: Address, validators: Vec<Validator>) -> Self {
        let genesis = Block::genesis();
        let mut decided = Vec::new();
        decided.push(genesis);
        
        Self {
            my_address,
            validators,
            utxo: UTXOEngine::new(),
            view: 0,
            decided_height: 0,
            prepare_certs: HashMap::new(),
            commit_certs: HashMap::new(),
            pending_txs: Vec::new(),
            decided_blocks: decided,
        }
    }
    
    /// Проверить, является ли адрес валидатором
    fn is_validator(&self, addr: &Address) -> bool {
        self.validators.iter().any(|v| &v.address == addr)
    }
    
    /// Получить лидера текущей view
    fn get_leader(&self, view: u64) -> &Validator {
        let idx = (view as usize) % self.validators.len();
        &self.validators[idx]
    }
    
    /// Размер кворума: 2f + 1
    fn quorum_size(&self) -> usize {
        let f = (self.validators.len() - 1) / 3;
        2 * f + 1
    }
    
    /// Добавить транзакцию в mempool
    pub fn add_transaction(&mut self, tx: Transaction) {
        self.pending_txs.push(tx);
        debug!("Added tx to mempool, size={}", self.pending_txs.len());
    }
    
    /// PROPOSE: Лидер создаёт блок
    pub fn propose(&mut self, view: u64) -> Result<Block, ConsensusError> {
        let leader = self.get_leader(view);
        if leader.address != self.my_address {
            return Err(ConsensusError::NotValidator);
        }
        
        // Take up to 64 transactions
        let batch_size = self.pending_txs.len().min(64);
        let txs: Vec<Transaction> = self.pending_txs.drain(..batch_size).collect();
        
        let block = self.utxo.create_block(txs, leader.address.clone());
        
        info!("PROPOSE view={} height={} txs={}", view, block.height, block.transactions.len());
        
        Ok(block)
    }
    
    /// PREPARE: Валидатор проверяет и голосует за блок
    pub fn prepare(&mut self, block: &Block, view: u64) -> Result<(), ConsensusError> {
        if !self.is_validator(&self.my_address) {
            return Err(ConsensusError::NotValidator);
        }
        
        // Basic validation
        if block.height != self.decided_height + 1 {
            return Err(ConsensusError::InvalidProposal);
        }
        
        // Check leader
        let leader = self.get_leader(view);
        if block.proposer != leader.address {
            return Err(ConsensusError::InvalidProposal);
        }
        
        // Validate transactions
        for tx in &block.transactions {
            if let Err(e) = self.utxo.spend(tx) {
                warn!("Invalid tx in block: {}", e);
                return Err(ConsensusError::UTXO(e));
            }
        }
        
        // Record prepare vote
        let cert = self.prepare_certs.entry(block.height).or_default();
        if !cert.insert(self.my_address.clone()) {
            return Err(ConsensusError::DoubleVote);
        }
        
        info!("PREPARE height={} votes={}/{}", 
              block.height, cert.len(), self.quorum_size());
        
        Ok(())
    }
    
    /// COMMIT: Достигнут кворум prepare -> голосуем за commit
    pub fn commit(&mut self, height: u64) -> Result<(), ConsensusError> {
        let cert = self.prepare_certs.get(&height)
            .ok_or(ConsensusError::QuorumNotReached)?;
        
        if cert.len() < self.quorum_size() {
            return Err(ConsensusError::QuorumNotReached);
        }
        
        // Record commit vote
        let commit_cert = self.commit_certs.entry(height).or_default();
        if !commit_cert.insert(self.my_address.clone()) {
            return Err(ConsensusError::DoubleVote);
        }
        
        info!("COMMIT height={} votes={}/{}", 
              height, commit_cert.len(), self.quorum_size());
        
        Ok(())
    }
    
    /// DECIDE: Кворум commit -> блок финализирован
    pub fn decide(&mut self, block: Block) -> Result<(), ConsensusError> {
        let height = block.height;
        let cert = self.commit_certs.get(&height)
            .ok_or(ConsensusError::QuorumNotReached)?;
        
        if cert.len() < self.quorum_size() {
            return Err(ConsensusError::QuorumNotReached);
        }
        
        // Finalize
        self.decided_height = height;
        self.decided_blocks.push(block.clone());
        self.view += 1;
        
        // Cleanup old certs
        self.prepare_certs.retain(|&h, _| h > height.saturating_sub(10));
        self.commit_certs.retain(|&h, _| h > height.saturating_sub(10));
        
        info!("DECIDE height={} total_blocks={}", height, self.decided_blocks.len());
        
        Ok(())
    }
    
    /// Полный цикл консенсуса (для тестирования)
    pub fn run_consensus_round(&mut self) -> Result<Block, ConsensusError> {
        let view = self.view;
        
        // Only leader proposes
        let block = match self.propose(view) {
            Ok(b) => b,
            Err(ConsensusError::NotValidator) => {
                // Not leader, wait for proposal
                warn!("Not leader in view {}, waiting...", view);
                return Err(ConsensusError::NotValidator);
            }
            Err(e) => return Err(e),
        };
        
        // Simulate all validators voting (for testing)
        for validator in &self.validators {
            // In real network: broadcast PREPARE msg
            let cert = self.prepare_certs.entry(block.height).or_default();
            cert.insert(validator.address.clone());
        }
        
        // Check prepare quorum
        self.commit(block.height)?;
        
        // Simulate all validators committing
        for validator in &self.validators {
            let cert = self.commit_certs.entry(block.height).or_default();
            cert.insert(validator.address.clone());
        }
        
        // Finalize
        self.decide(block.clone())?;
        
        Ok(block)
    }
    
    /// Получить статус
    pub fn status(&self) -> ConsensusStatus {
        ConsensusStatus {
            view: self.view,
            decided_height: self.decided_height,
            pending_txs: self.pending_txs.len(),
            validator_count: self.validators.len(),
            quorum: self.quorum_size(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConsensusStatus {
    pub view: u64,
    pub decided_height: u64,
    pub pending_txs: usize,
    pub validator_count: usize,
    pub quorum: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_validators(n: usize) -> Vec<Validator> {
        (0..n).map(|i| Validator {
            address: Address([(i as u8); 32]),
            stake: 10_000,
            name: format!("validator-{}", i),
        }).collect()
    }
    
    #[test]
    fn test_consensus_4_validators() {
        let validators = create_test_validators(4);
        let my_addr = validators[0].address.clone();
        let mut engine = ConsensusEngine::new(my_addr, validators);
        
        // Add some transactions
        for i in 0..10 {
            let tx = Transaction {
                nullifier: Nullifier([i as u8; 32]),
                inputs: vec![],
                outputs: vec![],
                fee: 0,
                proof: vec![],
            };
            engine.add_transaction(tx);
        }
        
        // Run consensus
        let block = engine.run_consensus_round().unwrap();
        assert_eq!(block.height, 1);
        assert_eq!(engine.decided_height, 1);
        
        // Quorum for 4 validators = 3 (f=1, 2f+1=3)
        assert_eq!(engine.status().quorum, 3);
    }
    
    #[test]
    fn test_not_leader() {
        let validators = create_test_validators(4);
        let my_addr = validators[3].address.clone();  // Not leader for view 0
        let mut engine = ConsensusEngine::new(my_addr, validators);
        
        assert!(matches!(
            engine.propose(0),
            Err(ConsensusError::NotValidator)
        ));
    }
}
'''

with open(os.path.join(base_dir, "src/consensus/mod.rs"), "w") as f:
    f.write(consensus_mod)

print("src/consensus/mod.rs created")
