
# src/crypto/mod.rs - криптографические примитивы
crypto_mod = '''//! Криптографические примитивы QRAP
//! 
//! Пока используем SHA3-256 вместо Poseidon (для скорости разработки).
//! В production заменить на Poseidon-256 (STARK-friendly).

use crate::types::Hash;
use sha3::{Digest, Sha3_256};

/// Хэширование с SHA3-256 (placeholder для Poseidon-256)
pub fn hash(data: &[u8]) -> Hash {
    let result = Sha3_256::digest(data);
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&result);
    Hash(bytes)
}

/// Двойное хэширование (аналог Bitcoin, защита от length extension)
pub fn hash_double(data: &[u8]) -> Hash {
    let first = Sha3_256::digest(data);
    let second = Sha3_256::digest(&first);
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&second);
    Hash(bytes)
}

/// Merkle root из списка листьев
pub fn merkle_root(leaves: &[Hash]) -> Hash {
    if leaves.is_empty() {
        return Hash([0u8; 32]);
    }
    
    let mut current_level: Vec<Hash> = leaves.to_vec();
    
    while current_level.len() > 1 {
        let mut next_level = Vec::new();
        
        for chunk in current_level.chunks(2) {
            let left = &chunk[0];
            let right = if chunk.len() > 1 { &chunk[1] } else { left };
            
            let mut combined = Vec::with_capacity(64);
            combined.extend_from_slice(left.as_bytes());
            combined.extend_from_slice(right.as_bytes());
            
            next_level.push(hash(&combined));
        }
        
        current_level = next_level;
    }
    
    current_level[0].clone()
}

/// Merkle path (siblings для верификации)
pub fn merkle_path(leaves: &[Hash], index: usize) -> Vec<(Hash, bool)> {
    // Returns: (sibling_hash, is_right)
    // true если текущий узел - левый, sibling - правый
    let mut path = Vec::new();
    let mut current_level: Vec<Hash> = leaves.to_vec();
    let mut current_index = index;
    
    while current_level.len() > 1 {
        let is_right = current_index % 2 == 0;
        let sibling_index = if is_right { current_index + 1 } else { current_index - 1 };
        
        if sibling_index < current_level.len() {
            path.push((current_level[sibling_index].clone(), is_right));
        } else {
            // Дублируем последний элемент если нечётное количество
            path.push((current_level[current_index].clone(), is_right));
        }
        
        current_index /= 2;
        
        // Build next level
        let mut next_level = Vec::new();
        for chunk in current_level.chunks(2) {
            let left = &chunk[0];
            let right = if chunk.len() > 1 { &chunk[1] } else { left };
            
            let mut combined = Vec::with_capacity(64);
            combined.extend_from_slice(left.as_bytes());
            combined.extend_from_slice(right.as_bytes());
            
            next_level.push(hash(&combined));
        }
        current_level = next_level;
    }
    
    path
}

/// Верификация Merkle path
pub fn verify_merkle_path(root: &Hash, leaf: &Hash, path: &[(Hash, bool)]) -> bool {
    let mut current = leaf.clone();
    
    for (sibling, is_right) in path {
        let mut combined = Vec::with_capacity(64);
        if *is_right {
            combined.extend_from_slice(current.as_bytes());
            combined.extend_from_slice(sibling.as_bytes());
        } else {
            combined.extend_from_slice(sibling.as_bytes());
            combined.extend_from_slice(current.as_bytes());
        }
        current = hash(&combined);
    }
    
    current == *root
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_merkle() {
        let leaves: Vec<Hash> = (0..4)
            .map(|i| hash(&i.to_le_bytes()))
            .collect();
        
        let root = merkle_root(&leaves);
        let path = merkle_path(&leaves, 2);
        
        assert!(verify_merkle_path(&root, &leaves[2], &path));
    }
}
'''

with open(os.path.join(base_dir, "src/crypto/mod.rs"), "w") as f:
    f.write(crypto_mod)

print("src/crypto/mod.rs created")
