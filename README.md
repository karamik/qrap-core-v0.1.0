# QRAP Core — Quantum-Resistant Anchor Protocol

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg)](https://www.rust-lang.org)
[![Status](https://img.shields.io/badge/status-research--grade-blue.svg)]()

> **Post-Quantum. Anonymous. Verifiable.**

QRAP is a research-grade implementation of a post-quantum anonymous blockchain protocol. It combines lattice-based cryptography (Ring-LWE), zero-knowledge STARK proofs, and epoch-based nullifier trees to build a foundation for quantum-resistant decentralized finance.

This is **v0.1.0** — a working proof-of-concept. Not production. Not audited. But mathematically grounded and open for collaboration.

---

## What Problem Does QRAP Solve?

| Threat | Current Blockchains | QRAP |
|--------|-------------------|------|
| **Quantum computers** breaking ECDSA/RSA | Vulnerable | **Resistant** — Ring-LWE + ML-DSA |
| **Transaction linkability** | Pseudonymous | **Anonymous** — ZK-STARK spend proofs |
| **Bridge hacks** | $2B+ stolen in 2022-2024 | **Eliminated** — atomic swaps via Pace protocol |
| **State bloat** | O(n) storage | **Bounded** — epoch-based nullifier trees |

---

## Architecture (v0.1.0)

```
┌─────────────────────────────────────────┐
│  L2: Recursive STARK Rollup             │
│  64-1024 txs → 1 proof (~200 KB)        │
│  Field: M31 → M31⁴ → Goldilocks         │
├─────────────────────────────────────────┤
│  L1: Orbital BFT Consensus              │
│  4 validators, 12s blocks, instant    │
│  finality, forced inclusion             │
├─────────────────────────────────────────┤
│  UTXO Engine + Epoch Nullifier Trees    │
│  Ring-LWE commitments, Poseidon-256     │
│  Sparse Merkle Trees, 100-block epochs  │
└─────────────────────────────────────────┘
```

---

## Quick Start

### Prerequisites

- Rust 1.80+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- 8 GB RAM minimum, 16 GB recommended
- Linux, macOS, or WSL2

### Build

```bash
git clone https://github.com/karamik/qrap-core-v0.1.0.git
cd qrap-core-v0.1.0
cargo build --release
```

### Run Tests

```bash
cargo test
```

### Benchmark

```bash
./target/release/qrap-node benchmark --txs 1000
```

### Start a Node

```bash
# Generate keys
./target/release/qrap-node keygen --output validator.keys

# Run validator
./target/release/qrap-node run --config config/qrap.toml
```

---

## What's Implemented (v0.1.0)

| Component | Status | Notes |
|-----------|--------|-------|
| UTXO engine with mint/spend | ✅ Working | Double-spend protection via nullifiers |
| Epoch Nullifier Trees (ENT) | ✅ Working | 100-block epochs, automatic rollover |
| Simplified BFT consensus | ✅ Working | Propose → Prepare → Commit → Decide |
| Merkle tree operations | ✅ Working | SHA3-256 (Poseidon placeholder) |
| CLI (keygen, run, benchmark) | ✅ Working | See `--help` |
| **Poseidon-256 hash** | 🔄 Placeholder | Uses SHA3-256 for now |
| **STARK prover** | 🔄 Placeholder | ZK-proof field is `vec![]` |
| **libp2p networking** | 🔄 Placeholder | Single-node only |
| **RocksDB persistence** | 🔄 Placeholder | In-memory storage |
| ML-DSA signatures | 📋 Specified | NIST FIPS 204 |
| ML-KEM key exchange | 📋 Specified | NIST FIPS 203 |

---

## What's Next

| Milestone | Target | Deliverable |
|-----------|--------|-------------|
| v0.2.0 | Q3 2026 | libp2p networking, 4-node testnet |
| v0.3.0 | Q4 2026 | Winterfell STARK prover integration |
| v0.4.0 | Q4 2026 | Poseidon-256, RocksDB persistence |
| v0.5.0 | Q1 2027 | Public testnet, faucet, explorer |
| v1.0.0 | Q2 2027 | Audited mainnet candidate |

---

## Research & Formal Verification

QRAP is designed to be formally verifiable:

- **TLA+**: Consensus safety and liveness proofs ([spec](docs/tla/))
- **ACL2**: ZK-circuit correctness via PFCS framework ([spec](docs/acl2/))
- **Event-B**: Epoch processing refinement ([spec](docs/eventb/))

See [TOTAL Protocol v8.2 Specification](docs/SPECIFICATION.md) for full mathematical treatment.

---

## Contributing

We need:

- **Rust developers** — libp2p, cryptography, database layers
- **Cryptographers** — lattice-based ZK circuits, STARK optimization
- **Formal verification engineers** — TLA+, ACL2, Coq
- **DevOps** — testnet infrastructure, monitoring, CI/CD

Open an issue or PR. All contributions are welcome.

---

## Citation

```bibtex
@software{qrap2026,
  title = {QRAP Core: Quantum-Resistant Anchor Protocol},
  author = {TOTAL Protocol Team},
  year = {2026},
  url = {https://github.com/karamik/qrap-core-v0.1.0},
  version = {0.1.0}
}
```

---

## License

MIT — see [LICENSE](LICENSE).

---

## Contact

- **Telegram:** [@tec_support_bot](https://t.me/tec_support_bot)
- **Issues:** [GitHub Issues](https://github.com/karamik/qrap-core-v0.1.0/issues)
- **Discussions:** [GitHub Discussions](https://github.com/karamik/qrap-core-v0.1.0/discussions)

> **In Physics We Trust.** Not hype. Not promises. Open code, open math, open hardware.
