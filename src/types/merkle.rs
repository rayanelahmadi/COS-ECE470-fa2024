use super::hash::{Hashable, H256};

/// A Merkle tree.
#[derive(Debug, Default)]
pub struct MerkleTree {
    nodes: Vec<Vec<H256>>, // A vector of vectors to represent tree levels
}

impl MerkleTree {
    pub fn new<T>(data: &[T]) -> Self where T: Hashable, {
        //unimplemented!()
        if data.is_empty() {
            return MerkleTree {
                nodes: vec![vec![H256::from([0u8; 32])]],
            };
        }

        let mut current_level: Vec<H256> = data.iter().map(|item| item.hash()).collect();
        let mut nodes = vec![current_level.clone()]; // Store all levels

        while current_level.len() > 1 {
            if current_level.len() % 2 == 1 {
                current_level.push(current_level[current_level.len() - 1]); // Duplicate last node if odd number
            }

            let mut next_level = Vec::new();
            for pair in current_level.chunks(2) {
                let concatenated = [pair[0].as_ref(), pair[1].as_ref()].concat();
                next_level.push(H256::from(ring::digest::digest(&ring::digest::SHA256, &concatenated)));
            }

            nodes.push(next_level.clone());
            current_level = next_level;
        }

        MerkleTree { nodes }
    }

    pub fn root(&self) -> H256 {
        //unimplemented!()
        if let Some(root_level) = self.nodes.last() {
            return root_level[0]; // Root is the only element in the top level
        }
        // Return an inline zero-filled H256 hash if tree is empty 
        H256::from([0u8;32])
    }

    /// Returns the Merkle Proof of data at index i
    pub fn proof(&self, index: usize) -> Vec<H256> {
        //unimplemented!()
        let mut proof = Vec::new();
        let mut current_index = index;

        for level in &self.nodes[..self.nodes.len() - 1] {
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };

            if sibling_index < level.len() {
                proof.push(level[sibling_index]);
            }

            current_index /= 2; // Move up one level 

        }

        proof
    }
}

/// Verify that the datum hash with a vector of proofs will produce the Merkle root. Also need the
/// index of datum and `leaf_size`, the total number of leaves.
pub fn verify(root: &H256, datum: &H256, proof: &[H256], index: usize, leaf_size: usize) -> bool {
    //unimplemented!()
    if index >= leaf_size {
        return false;
    }

    let mut hash = *datum;
    let mut index = index;
    
    for sibling_hash in proof {
        if index % 2 == 0 {
            hash = H256::from(ring::digest::digest(
                &ring::digest::SHA256,
                &[hash.as_ref(), sibling_hash.as_ref()].concat(),
            ));

        } else {
            hash = H256::from(ring::digest::digest(
                &ring::digest::SHA256,
                &[sibling_hash.as_ref(), hash.as_ref()].concat(),
            ));
            
        }
        index /= 2;
    }
    &hash == root
    
}
// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use crate::types::hash::H256;
    use super::*;

    macro_rules! gen_merkle_tree_data {
        () => {{
            vec![
                (hex!("0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d")).into(),
                (hex!("0101010101010101010101010101010101010101010101010101010101010202")).into(),
            ]
        }};
    }

    #[test]
    fn merkle_root() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let root = merkle_tree.root();
        assert_eq!(
            root,
            (hex!("6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920")).into()
        );
        // "b69566be6e1720872f73651d1851a0eae0060a132cf0f64a0ffaea248de6cba0" is the hash of
        // "0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d"
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
        // "6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920" is the hash of
        // the concatenation of these two hashes "b69..." and "965..."
        // notice that the order of these two matters
    }

    #[test]
    fn merkle_proof() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert_eq!(proof,
                   vec![hex!("965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f").into()]
        );
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
    }

    #[test]
    fn merkle_verifying() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert!(verify(&merkle_tree.root(), &input_data[0].hash(), &proof, 0, input_data.len()));
    }

    #[test]
    fn merkle_tree_empty() {
        let input_data: Vec<H256> = vec![];
        let merkle_tree = MerkleTree::new(&input_data);
        let root = merkle_tree.root();
        
        // Root should be a 32-byte array of zeros
        assert_eq!(root, H256::from([0u8; 32]));
        
    }

    #[test]
    fn merkle_tree_single_element() {
        let input_data: Vec<H256> = vec![
            (hex!("0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d")).into(),
        ];
        let merkle_tree = MerkleTree::new(&input_data);
        let root = merkle_tree.root();
        // Root should be the hash of the single element
        assert_eq!(root, input_data[0].hash());
    }

    #[test]
    fn merkle_tree_odd_number_of_elements() {
        let input_data: Vec<H256> = vec![
            (hex!("0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d")).into(),
            (hex!("0101010101010101010101010101010101010101010101010101010101010202")).into(),
            (hex!("deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef")).into(),
        ];
        let merkle_tree = MerkleTree::new(&input_data);
        let root = merkle_tree.root();
        // We don't know the exact root value but make sure it doesn't panic
        assert_ne!(root, H256::from([0u8; 32])); // Root should not be zero
    }

    #[test]
    fn merkle_tree_proof_out_of_bounds() {
        let input_data: Vec<H256> = vec![
            (hex!("0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d")).into(),
            (hex!("0101010101010101010101010101010101010101010101010101010101010202")).into(),
        ];
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(2); // Index 2 is out of bounds
        assert_eq!(proof.len(), 0); // Proof should return an empty vector
    }

    #[test]
    fn merkle_verifying_invalid_proof() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let mut proof = merkle_tree.proof(0);
        
        // Corrupt the proof by altering one of the hashes
        proof[0] = H256::from([0xff; 32]);

        // Verifying should fail with a corrupted proof
        assert!(!verify(&merkle_tree.root(), &input_data[0].hash(), &proof, 0, input_data.len()));
    }

    #[test]
    fn merkle_verifying_incorrect_index() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);

        // Verify the proof using an incorrect index (1 instead of 0)
        assert!(!verify(&merkle_tree.root(), &input_data[0].hash(), &proof, 1, input_data.len()));
    }

}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST