
# src/main.rs - CLI + запуск ноды
main_rs = '''//! QRAP Node CLI
//! 
//! Запуск: cargo run -- --help

use clap::{Parser, Subcommand};
use qrap_core::*;
use tracing::{info, error};

#[derive(Parser)]
#[command(name = "qrap-node")]
#[command(about = "QRAP - Quantum-Resistant Anchor Protocol Node")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a QRAP node
    Run {
        /// Path to configuration file
        #[arg(short, long, default_value = "~/.qrap/config/qrap.toml")]
        config: String,
    },
    
    /// Generate validator keys
    Keygen {
        /// Output file for keys
        #[arg(short, long, default_value = "validator.keys")]
        output: String,
        
        /// Validator name
        #[arg(short, long)]
        name: Option<String>,
    },
    
    /// Check node status
    Status,
    
    /// Initialize genesis state
    InitGenesis {
        /// Number of validators
        #[arg(short, long, default_value = "4")]
        validators: usize,
        
        /// Output file
        #[arg(short, long, default_value = "genesis.json")]
        output: String,
    },
    
    /// Run benchmark
    Benchmark {
        /// Number of transactions
        #[arg(short, long, default_value = "1000")]
        txs: usize,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("qrap_core=info".parse()?))
        .init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Run { config } => {
            info!("Starting QRAP node v{}", PROTOCOL_VERSION);
            info!("Config: {}", config);
            
            let config = NodeConfig {
                node_type: NodeType::Validator,
                name: "qrap-node".to_string(),
                listen_addr: "0.0.0.0:30303".to_string(),
                data_dir: "~/.qrap/data".to_string(),
            };
            
            run_node(config).await?;
        }
        
        Commands::Keygen { output, name } => {
            let name = name.unwrap_or_else(|| "validator".to_string());
            info!("Generating keys for: {}", name);
            
            // Generate placeholder keys
            use rand::RngCore;
            let mut secret = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut secret);
            
            let address = types::Address::from_pubkey(&secret);
            
            println!("Validator: {}", name);
            println!("Address: {:?}", address);
            println!("Keys saved to: {}", output);
            
            // Save to file
            std::fs::write(&output, hex::encode(&secret))?;
        }
        
        Commands::Status => {
            println!("QRAP Node Status");
            println!("================");
            println!("Version: {}", PROTOCOL_VERSION);
            println!("Chain ID: {}", TESTNET_CHAIN_ID);
            println!("Protocol: Orbital BFT + Ring-LWE");
            println!("Status: Not running (use 'run' to start)");
        }
        
        Commands::InitGenesis { validators, output } => {
            info!("Initializing genesis with {} validators", validators);
            
            let genesis = types::Block::genesis();
            let json = serde_json::to_string_pretty(&genesis)?;
            std::fs::write(&output, json)?;
            
            println!("Genesis block created: {}", output);
            println!("Validators: {}", validators);
        }
        
        Commands::Benchmark { txs } => {
            info!("Running benchmark with {} transactions", txs);
            
            use qrap_core::utxo::*;
            use qrap_core::types::*;
            use std::time::Instant;
            
            let mut engine = UTXOEngine::new();
            let owner = Address([1u8; 32]);
            let secret = b"benchmark_secret";
            
            // Mint phase
            let start = Instant::now();
            let mut utxos = Vec::new();
            for i in 0..txs {
                let utxo = engine.mint(owner.clone(), 1000 + i as u64, secret).unwrap();
                utxos.push(utxo);
            }
            let mint_time = start.elapsed();
            
            // Spend phase
            let start = Instant::now();
            for (i, utxo) in utxos.iter().enumerate() {
                let nullifier = Nullifier::derive(secret, utxo.coin_id, utxo.epoch_created);
                let output = Commitment::new(&format!("secret{}", i).into_bytes(), &[0u8; 32], 900);
                
                let tx = Transaction {
                    nullifier,
                    inputs: vec![utxo.clone()],
                    outputs: vec![output],
                    fee: 100,
                    proof: vec![],
                };
                
                engine.spend(&tx).unwrap();
            }
            let spend_time = start.elapsed();
            
            println!("Benchmark Results");
            println!("=================");
            println!("Transactions: {}", txs);
            println!("Mint time: {:?} ({:.2} tx/s)", mint_time, txs as f64 / mint_time.as_secs_f64());
            println!("Spend time: {:?} ({:.2} tx/s)", spend_time, txs as f64 / spend_time.as_secs_f64());
            println!("Total time: {:?}", mint_time + spend_time);
            
            let stats = engine.stats();
            println!("Final state: {:?}", stats);
        }
    }
    
    Ok(())
}

mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}
'''

with open(os.path.join(base_dir, "src/main.rs"), "w") as f:
    f.write(main_rs)

# Добавляем hex в lib.rs
lib_rs_updated = '''pub mod types;
pub mod crypto;
pub mod utxo;
pub mod consensus;
pub mod network;
pub mod storage;

use tracing::{info, warn};

pub const PROTOCOL_VERSION: &str = "0.1.0-testnet";
pub const TESTNET_CHAIN_ID: u64 = 170206;
pub const EPOCH_SIZE: u64 = 100;
pub const BLOCK_TIME_SECS: u64 = 12;
pub const MIN_STAKE: u64 = 10_000;

#[derive(Debug, Clone)]
pub struct NodeConfig {
    pub node_type: NodeType,
    pub name: String,
    pub listen_addr: String,
    pub data_dir: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Validator,
    FullNode,
    Prover,
    Archival,
}

pub async fn run_node(config: NodeConfig) -> anyhow::Result<()> {
    info!("Starting QRAP node v{}", PROTOCOL_VERSION);
    info!("Chain ID: {}", TESTNET_CHAIN_ID);
    info!("Node type: {:?}", config.node_type);
    
    // Placeholder - full implementation in v0.2
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
}
'''

with open(os.path.join(base_dir, "src/lib.rs"), "w") as f:
    f.write(lib_rs_updated)

print("src/main.rs created")
print("src/lib.rs updated")
