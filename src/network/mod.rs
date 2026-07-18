
# Создаём placeholder модули network и storage
# src/network/mod.rs
network_mod = '''//! P2P Network layer (placeholder)
//! 
//! В v0.1: заглушка. В v0.2: libp2p интеграция

use tracing::info;

pub struct NetworkLayer;

impl NetworkLayer {
    pub fn new() -> Self {
        info!("Network layer initialized (placeholder)");
        Self
    }
}
'''

with open(os.path.join(base_dir, "src/network/mod.rs"), "w") as f:
    f.write(network_mod)

# src/storage/mod.rs
storage_mod = '''//! Storage layer (placeholder)
//! 
//! В v0.1: in-memory. В v0.2: RocksDB backend

use crate::types::*;
use std::collections::HashMap;

pub struct Storage {
    blocks: HashMap<u64, Block>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
        }
    }
    
    pub fn save_block(&mut self, block: Block) {
        self.blocks.insert(block.height, block);
    }
    
    pub fn get_block(&self, height: u64) -> Option<&Block> {
        self.blocks.get(&height)
    }
}
'''

with open(os.path.join(base_dir, "src/storage/mod.rs"), "w") as f:
    f.write(storage_mod)

# benches/utxo_benchmark.rs
bench_rs = '''use criterion::{black_box, criterion_group, criterion_main, Criterion};
use qrap_core::utxo::*;
use qrap_core::types::*;

fn benchmark_mint(c: &mut Criterion) {
    let mut engine = UTXOEngine::new();
    let owner = Address([1u8; 32]);
    let secret = b"benchmark";
    
    c.bench_function("mint_utxo", |b| {
        b.iter(|| {
            engine.mint(owner.clone(), black_box(1000), secret).unwrap();
        });
    });
}

fn benchmark_spend(c: &mut Criterion) {
    let mut engine = UTXOEngine::new();
    let owner = Address([1u8; 32]);
    let secret = b"benchmark";
    
    // Pre-mint
    let utxo = engine.mint(owner.clone(), 1000, secret).unwrap();
    
    c.bench_function("spend_utxo", |b| {
        b.iter(|| {
            let nullifier = Nullifier::derive(secret, utxo.coin_id, utxo.epoch_created);
            let output = Commitment::new(b"new", &[0u8; 32], 900);
            
            let tx = Transaction {
                nullifier,
                inputs: vec![utxo.clone()],
                outputs: vec![output],
                fee: 100,
                proof: vec![],
            };
            
            engine.spend(&tx).unwrap();
        });
    });
}

criterion_group!(benches, benchmark_mint, benchmark_spend);
criterion_main!(benches);
'''

with open(os.path.join(base_dir, "benches/utxo_benchmark.rs"), "w") as f:
    f.write(bench_rs)

# tests/integration_test.rs
integration_test = '''//! Integration tests for QRAP Core

use qrap_core::*;
use qrap_core::types::*;
use qrap_core::utxo::*;
use qrap_core::consensus::*;

#[test]
fn test_full_flow() {
    // Setup
    let validators = vec![
        Validator {
            address: Address([1u8; 32]),
            stake: 10_000,
            name: "v1".to_string(),
        },
        Validator {
            address: Address([2u8; 32]),
            stake: 10_000,
            name: "v2".to_string(),
        },
        Validator {
            address: Address([3u8; 32]),
            stake: 10_000,
            name: "v3".to_string(),
        },
        Validator {
            address: Address([4u8; 32]),
            stake: 10_000,
            name: "v4".to_string(),
        },
    ];
    
    let my_addr = validators[0].address.clone();
    let mut consensus = ConsensusEngine::new(my_addr, validators);
    
    // Add transactions
    for i in 0..10 {
        let tx = Transaction {
            nullifier: Nullifier([i as u8; 32]),
            inputs: vec![],
            outputs: vec![],
            fee: 0,
            proof: vec![],
        };
        consensus.add_transaction(tx);
    }
    
    // Run consensus
    let block = consensus.run_consensus_round().unwrap();
    assert_eq!(block.height, 1);
    
    let status = consensus.status();
    assert_eq!(status.decided_height, 1);
    assert_eq!(status.validator_count, 4);
    assert_eq!(status.quorum, 3);  // f=1, 2f+1=3
}

#[test]
fn test_epoch_rollover() {
    let validators = vec![
        Validator {
            address: Address([1u8; 32]),
            stake: 10_000,
            name: "v1".to_string(),
        },
    ];
    
    let mut engine = UTXOEngine::new();
    let owner = Address([1u8; 32]);
    
    // Create blocks to trigger rollover
    for _ in 0..EPOCH_SIZE + 1 {
        engine.create_block(vec![], owner.clone());
    }
    
    let stats = engine.stats();
    assert_eq!(stats.epoch, 1);
    assert_eq!(stats.archived_epochs, 2);  // genesis + epoch 0
}
'''

with open(os.path.join(base_dir, "tests/integration_test.rs"), "w") as f:
    f.write(integration_test)

print("All modules created:")
print("  src/network/mod.rs")
print("  src/storage/mod.rs")
print("  benches/utxo_benchmark.rs")
print("  tests/integration_test.rs")
