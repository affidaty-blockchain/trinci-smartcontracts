// This file is part of TRINCI.
//
// Copyright (C) 2021 Affidaty Spa.
//
// TRINCI is free software: you can redistribute it and/or modify it under
// the terms of the GNU Affero General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at your
// option) any later version.
//
// TRINCI is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
// FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License
// for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with TRINCI. If not, see <https://www.gnu.org/licenses/>.

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use trinci_sdk::{WasmError, WasmResult};

/// Create sha256 hash from raw binary data
fn sha256(data: &[u8]) -> Vec<u8> {
    // create a Sha256 object
    let mut hasher = Sha256::new();

    // write input message
    hasher.update(&data);

    // read hash digest and consume hasher
    hasher.finalize().to_vec()
}

/// Create proof indices from leaves indices
pub fn get_proof_indices(tree_indices: &[i32], depth: i32) -> Vec<i32> {
    let leaf_count = 2i32.pow(depth as u32);
    let mut maximal_indices = Vec::<i32>::new();
    for index in tree_indices {
        let mut x = leaf_count + index;
        while x > 1 {
            maximal_indices.push(x ^ 1);
            x /= 2
        }
    }

    let a: Vec<i32> = tree_indices
        .iter()
        .map(|index| leaf_count + index)
        .collect();

    let mut b = maximal_indices.clone();
    b.sort_unstable();
    b.reverse();

    maximal_indices = [a, b].concat();

    let mut redundant_indices = Vec::<i32>::new();

    let mut proof: Vec<i32> = Vec::new();

    for mut index in maximal_indices {
        if !redundant_indices.contains(&index) {
            proof.push(index);
            while index > 1 {
                redundant_indices.push(index);
                if !redundant_indices.contains(&(index ^ 1)) {
                    break;
                }
                index /= 2;
            }
        }
    }

    proof
        .iter()
        .filter(|&index| !tree_indices.contains(&(index - leaf_count)))
        .copied()
        .collect()
}

/// Verify merkle tree data with multiproof
pub fn verify_merkle_tree_multiproof(
    root: &str,
    indices: &[i32],
    leaves: &[&str],
    depth: i32,
    proofs: &[&str],
) -> WasmResult<()> {
    let root = hex::decode(&root).map_err(|_| WasmError::new("invalid `root` hex"))?;

    let leaves = match leaves
        .iter()
        .map(|&x| hex::decode(x).map_err(|_| WasmError::new("error in leaves hex")))
        .collect::<WasmResult<Vec<Vec<u8>>>>()
    {
        Ok(val) => val,
        Err(e) => return Err(e),
    };
    let proofs = match proofs
        .iter()
        .map(|&x| hex::decode(x).map_err(|_| WasmError::new("error in proofs hex")))
        .collect::<WasmResult<Vec<Vec<u8>>>>()
    {
        Ok(val) => val,
        Err(e) => return Err(e),
    };

    let mut tree: BTreeMap<i32, Vec<u8>> = BTreeMap::new();

    indices.iter().zip(leaves).for_each(|(&index, leaf)| {
        tree.insert(2i32.pow(depth as u32) + index, leaf);
    });

    let proof_indices = get_proof_indices(indices, depth);

    proof_indices
        .iter()
        .zip(proofs)
        .for_each(|(&index, proof_item)| {
            tree.insert(index, proof_item);
        });

    let mut indexqueue: Vec<i32> = tree.keys().cloned().collect();
    indexqueue.sort_unstable();

    let mut indexqueue = indexqueue[0..(indexqueue.len() - 1)].to_vec();

    let mut i = 0;
    while i < indexqueue.len() {
        let index = indexqueue[i];
        if index >= 2 && tree.contains_key(&(index ^ 1)) {
            let pair = [
                tree[&(index - (index % 2))].clone(),
                tree[&(index - (index % 2) + 1)].clone(),
            ]
            .concat();
            tree.insert(index / 2, sha256(&pair));
            indexqueue.push(index / 2);
        }
        i += 1;
    }

    if !(indices.is_empty()
        || match tree.get(&1) {
            Some(root_value) => root_value == &root,
            None => return Err(WasmError::new("invalid leaves")),
        })
    {
        return Err(WasmError::new("invalid leaves"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_merkle_tree_multiproof_1() {
        let root = "f4d7d35867fb16f0399a60046a0d9f93a111adc50ab6c09c57c4885e0cba25a7";
        let indices = [1, 3];
        let leaves = [
            "4878a1d5f33d18548fb35e16323a22cb3cf81f6007c1c9810ffdb39c02ba6340",
            "4b76bbf06c32ab4dc45c13cb1c50a98d94d8fdee6a6284b8f52302a599dcaf0e",
        ];
        let depth = 3;

        let proofs = [
            "f1aff4a340342b07481cb87423a167c6955022668815f0801d4be37d8e50397f",
            "38c4ad2cb0b6cffba3b842dac4ba2e74e6685a65ed9460d24bea8fc6aa1dcaf4",
            "f9637951f10bd9e355419b1f84fd9efba0c8abcf5fd8e8381e46fe856482ee6e",
        ];

        assert!(verify_merkle_tree_multiproof(root, &indices, &leaves, depth, &proofs).is_ok());
    }

    #[test]
    fn test_verify_merkle_tree_multiproof_2() {
        let root = "57642fbe91a4710080ef401b7f1f15aa77a16fea85a5f757deaa215e05b2733a";
        let indices = [1, 3];
        let leaves = [
            "a3b3c1ae3b803953d5b89e67e25802a38f525fe8ff447914e67794f06ac6d87b",
            "40846390c66a0d1a77bd22513a2a51cd6a4ff597b1bec4d5e73bd8cf736e12b1",
        ];
        let depth = 3;

        let proofs = [
            "4b87b4a516220f0429483b18eaac96a332a5045a41e34346dd7a15489812d82b",
            "350f6198a9425ce4945837b9ae88863177e9c04dcea2c0be9acfbb86eb9a8d58",
            "e5fbae86ba26e1f9b3e16bd8761d88701451c45e2ba2686e75582b1ae3db41ad",
        ];

        assert!(verify_merkle_tree_multiproof(root, &indices, &leaves, depth, &proofs).is_ok());
    }

    #[test]
    fn test_verify_merkle_tree_multiproof_bad_1() {
        let root = "BADdd35867fb16f0399a60046a0d9f93a111adc50ab6c09c57c4885e0cba25a7"; // <-BAD
        let indices = [1, 3];
        let leaves = [
            "4878a1d5f33d18548fb35e16323a22cb3cf81f6007c1c9810ffdb39c02ba6340",
            "4b76bbf06c32ab4dc45c13cb1c50a98d94d8fdee6a6284b8f52302a599dcaf0e",
        ];
        let depth = 3;

        let proofs = [
            "f1aff4a340342b07481cb87423a167c6955022668815f0801d4be37d8e50397f",
            "38c4ad2cb0b6cffba3b842dac4ba2e74e6685a65ed9460d24bea8fc6aa1dcaf4",
            "f9637951f10bd9e355419b1f84fd9efba0c8abcf5fd8e8381e46fe856482ee6e",
        ];

        assert_eq!(
            verify_merkle_tree_multiproof(root, &indices, &leaves, depth, &proofs)
                .unwrap_err()
                .to_string(),
            "invalid leaves"
        );
    }

    #[test]
    fn test_verify_merkle_tree_multiproof_bad_2() {
        let root = "57642fbe91a4710080ef401b7f1f15aa77a16fea85a5f757deaa215e05b2733a";
        let indices = [1, 3];
        let leaves = [
            "a3b3c1ae3b804953d5b89e67e25802a38f525fe8ff447914e67794f06ac6d87b", // <-- BAD
            "40846390c66a0d1a77bd22513a2a51cd6a4ff597b1bec4d5e73bd8cf736e12b1",
        ];
        let depth = 3;

        let proofs = [
            "4b87b4a516220f0429483b18eaac96a332a5045a41e34346dd7a15489812d82b",
            "350f6198a9425ce4945837b9ae88863177e9c04dcea2c0be9acfbb86eb9a8d58",
            "e5fbae86ba26e1f9b3e16bd8761d88701451c45e2ba2686e75582b1ae3db41ad",
        ];

        assert_eq!(
            verify_merkle_tree_multiproof(root, &indices, &leaves, depth, &proofs)
                .unwrap_err()
                .to_string(),
            "invalid leaves"
        );
    }

    #[test]
    fn test_verify_merkle_tree_multiproof_bad_3() {
        let root = "57642fbe91a4710080ef401b7f1f15aa77a16fea85a5f757deaa215e05b2733a";
        let indices = [1, 3];
        let leaves = [
            "a3b3c1ae3b803953d5b89e67e25802a38f525fe8ff447914e67794f06ac6d87b",
            "40846390c66a0d1a77bd22513a2a51cd6a4ff597b1bec4d5e73bd8cf736e12b1",
        ];
        let depth = 3;

        let proofs = [
            "4b87b4a516220f0429483b18eaac96a332a5045a41e34346dd7a15489812d82b",
            "350f6198a9425ce4945837b9ae88863177e9c04dcea2c0be9acfbb86eb9a8d57", // <-- BAD
            "e5fbae86ba26e1f9b3e16bd8761d88701451c45e2ba2686e75582b1ae3db41ad",
        ];

        assert_eq!(
            verify_merkle_tree_multiproof(root, &indices, &leaves, depth, &proofs)
                .unwrap_err()
                .to_string(),
            "invalid leaves"
        );
    }

    #[test]
    fn test_verify_merkle_tree_multiproof_bad_4() {
        let root = "57642fbe91a4710080ef401b7f1f15aa77a16fea85a5f757deaa215e05b2733a";
        let indices = [1, 3];
        let leaves = [
            "a3b3c1ae3b803953d5b89e67e25802a38f525fe8ff447914e67794f06ac6d87b",
            "40846390c66a0d1a77bd22513a2a51cd6a4ff597b1bec4d5e73bd8cf736e12b1",
        ];
        let depth = 3;

        let proofs = [
            "4b87b4a516220f0429483b18eaac96a332a5045a41e34346dd7a15489812d82b",
            "350f6198a9425ce4945837b9aeX8863177e9c04dcea2c0be9acfbb86eb9a8d58", // <-- Invalid HEX
            "e5fbae86ba26e1f9b3e16bd8761d88701451c45e2ba2686e75582b1ae3db41ad",
        ];

        assert_eq!(
            verify_merkle_tree_multiproof(root, &indices, &leaves, depth, &proofs)
                .unwrap_err()
                .to_string(),
            "error in proofs hex"
        );
    }

    #[test]
    fn test_get_proof_indices() {
        assert_eq!(
            get_proof_indices(&[1, 3, 10], 12),
            [4107, 4098, 4096, 2052, 1027, 1025, 257, 129, 65, 33, 17, 9, 5, 3]
        );
    }

    #[test]
    fn test_sha256() {
        let expected = "039058c6f2c0cb492c533b0a4d14ef77cc0f78abccced5287d84a1a2011cfb81";

        assert_eq!(sha256(&[1, 2, 3]), hex::decode(expected).unwrap());
    }
}
