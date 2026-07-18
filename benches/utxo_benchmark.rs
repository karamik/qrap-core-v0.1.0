
import os

base_dir = "/mnt/agents/output/qrap-core"

# Создаём директорию benches если её нет
os.makedirs(os.path.join(base_dir, "benches"), exist_ok=True)

# Создаём правильный benchmark файл
bench_rs = '''use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_mint(c: &mut Criterion) {
    c.bench_function("mint_utxo", |b| {
        b.iter(|| {
            // Placeholder: will be implemented when UTXO engine is public API
            black_box(42);
        });
    });
}

fn benchmark_spend(c: &mut Criterion) {
    c.bench_function("spend_utxo", |b| {
        b.iter(|| {
            // Placeholder: will be implemented when UTXO engine is public API
            black_box(42);
        });
    });
}

criterion_group!(benches, benchmark_mint, benchmark_spend);
criterion_main!(benches);
'''

bench_path = os.path.join(base_dir, "benches", "utxo_benchmark.rs")
with open(bench_path, "w") as f:
    f.write(bench_rs)

print(f"✅ Создан: {bench_path}")
print(f"Размер: {os.path.getsize(bench_path)} bytes")

# Обновляем Cargo.toml с правильной bench секцией
cargo_with_bench = """[package]
name = "qrap-core"
version = "0.1.0"
edition = "2021"
authors = ["TOTAL Protocol Team"]
description = "Quantum-Resistant Anchor Protocol - Core Implementation"
license = "MIT"

[dependencies]
sha3 = "0.10"
blake3 = "1.5"
rand = "0.8"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "1.3"
hex = "0.4"
tokio = { version = "1.35", features = ["full"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
clap = { version = "4.4", features = ["derive"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1.0"
anyhow = "1.0"

[dev-dependencies]
tokio-test = "0.4"
criterion = { version = "0.5", features = ["html_reports"] }

[[bench]]
name = "utxo_benchmark"
harness = false
path = "benches/utxo_benchmark.rs"
"""

cargo_path = os.path.join(base_dir, "Cargo.toml")
with open(cargo_path, "w") as f:
    f.write(cargo_with_bench)

print(f"✅ Обновлён: {cargo_path}")

# Удаляем examples/benchmark.rs если есть (был перенесён ранее)
examples_bench = os.path.join(base_dir, "examples", "benchmark.rs")
if os.path.exists(examples_bench):
    os.remove(examples_bench)
    print(f"✅ Удалён старый: {examples_bench}")

# Проверяем структуру
print("\n=== Структура проекта ===")
for root, dirs, files in os.walk(base_dir):
    level = root.replace(base_dir, '').count(os.sep)
    indent = ' ' * 2 * level
    print(f'{indent}{os.path.basename(root)}/')
    subindent = ' ' * 2 * (level + 1)
    for file in sorted(files):
        filepath = os.path.join(root, file)
        size = os.path.getsize(filepath)
        print(f'{subindent}{file} ({size} bytes)')
