use std::fmt::Error;

use light_hasher::{Hasher, Poseidon};
use light_merkle_tree_reference::MerkleTree;
use light_utils::bigint::bigint_to_be_bytes_array;
use reqwest::Client;

use crate::{
    batch_append_with_proofs::get_batch_append_with_proofs_inputs,
    batch_append_with_subtrees::calculate_hash_chain,
    batch_update::get_batch_update_inputs,
    gnark::{
        batch_append_with_proofs_json_formatter::BatchAppendWithProofsInputsJson,
        batch_update_json_formatter::update_inputs_string,
        constants::{PROVE_PATH, SERVER_ADDRESS},
        proof_helpers::{compress_proof, deserialize_gnark_proof_json, proof_from_json_struct},
    },
};

// TODO: rename to MockBatchedForester
pub struct MockBatchedForester<const HEIGHT: usize> {
    pub merkle_tree: MerkleTree<Poseidon>,
    pub input_queue_leaves: Vec<([u8; 32], usize)>,
    /// Indices of leaves which in merkle tree which are active.
    pub output_queue_leaves: Vec<[u8; 32]>,
    pub active_leaves: Vec<[u8; 32]>,
    pub tx_events: Vec<MockTxEvent>,
}

#[derive(Debug, Clone)]
pub struct MockTxEvent {
    pub tx_hash: [u8; 32],
    pub inputs: Vec<[u8; 32]>,
    pub outputs: Vec<[u8; 32]>,
}

impl<const HEIGHT: usize> Default for MockBatchedForester<HEIGHT> {
    fn default() -> Self {
        let merkle_tree = MerkleTree::<Poseidon>::new(HEIGHT, 0);
        let input_queue_leaves = vec![];
        Self {
            merkle_tree,
            input_queue_leaves,
            output_queue_leaves: vec![],
            active_leaves: vec![],
            tx_events: vec![],
        }
    }
}

impl<const HEIGHT: usize> MockBatchedForester<HEIGHT> {
    pub async fn get_batched_append_proof(
        &mut self,
        account_next_index: usize,
        num_zkp_updates: u32,
        batch_size: u32,
        leaves_hashchain: [u8; 32],
        max_num_zkp_updates: u32,
    ) -> Result<(CompressedProof, [u8; 32]), Error> {
        let leaves = self.output_queue_leaves.to_vec();
        let start = num_zkp_updates as usize * batch_size as usize;
        let end = start + batch_size as usize;
        let leaves = leaves[start..end].to_vec();
        // if batch is complete, remove leaves from mock output queue
        if num_zkp_updates == max_num_zkp_updates - 1 {
            for _ in 0..max_num_zkp_updates * batch_size {
                self.output_queue_leaves.remove(0);
            }
        }
        let local_leaves_hashchain = calculate_hash_chain(&leaves);
        assert_eq!(leaves_hashchain, local_leaves_hashchain);
        let old_root = self.merkle_tree.root();
        let mut old_leaves = vec![];
        let mut merkle_proofs = vec![];
        for i in account_next_index..account_next_index + batch_size as usize {
            match self.merkle_tree.get_leaf(i) {
                Ok(leaf) => {
                    old_leaves.push(leaf);
                }
                Err(_) => {
                    old_leaves.push([0u8; 32]);
                    if i <= self.merkle_tree.get_next_index() {
                        self.merkle_tree.append(&[0u8; 32]).unwrap();
                    }
                }
            }
            let proof = self.merkle_tree.get_proof_of_leaf(i, true).unwrap();
            merkle_proofs.push(proof.to_vec());
        }
        // Insert new leaves into the merkle tree. Every leaf which is not [0u8;
        // 32] has already been nullified hence shouldn't be updated.
        for (i, leaf) in leaves.iter().enumerate() {
            if old_leaves[i] == [0u8; 32] {
                let index = account_next_index + i;
                self.merkle_tree.update(leaf, index).unwrap();
            }
        }
        let circuit_inputs = get_batch_append_with_proofs_inputs::<HEIGHT>(
            old_root,
            account_next_index as u32,
            leaves,
            local_leaves_hashchain,
            old_leaves,
            merkle_proofs,
            batch_size,
        );
        assert_eq!(
            bigint_to_be_bytes_array::<32>(&circuit_inputs.new_root.to_biguint().unwrap()).unwrap(),
            self.merkle_tree.root()
        );
        let client = Client::new();
        let inputs_json = BatchAppendWithProofsInputsJson::from_inputs(&circuit_inputs).to_string();

        let response_result = client
            .post(&format!("{}{}", SERVER_ADDRESS, PROVE_PATH))
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(inputs_json)
            .send()
            .await
            .expect("Failed to execute request.");
        if response_result.status().is_success() {
            let body = response_result.text().await.unwrap();
            let proof_json = deserialize_gnark_proof_json(&body).unwrap();
            let (proof_a, proof_b, proof_c) = proof_from_json_struct(proof_json);
            let (proof_a, proof_b, proof_c) = compress_proof(&proof_a, &proof_b, &proof_c);
            return Ok((
                CompressedProof {
                    a: proof_a,
                    b: proof_b,
                    c: proof_c,
                },
                bigint_to_be_bytes_array::<32>(&circuit_inputs.new_root.to_biguint().unwrap())
                    .unwrap(),
            ));
        }
        Err(Error)
    }

    pub async fn get_batched_update_proof(
        &mut self,
        batch_size: u32,
        leaves_hashchain: [u8; 32],
    ) -> Result<(CompressedProof, [u8; 32]), Error> {
        let mut merkle_proofs = vec![];
        let mut path_indices = vec![];
        let leaves = self.input_queue_leaves[..batch_size as usize].to_vec();
        let old_root = self.merkle_tree.root();
        let mut nullifiers = Vec::new();
        let mut tx_hashes = Vec::new();
        let mut old_leaves = Vec::new();
        for (leaf, index) in leaves.iter() {
            let index = *index;
            // + 2 because next index is + 1 and we need to init the leaf in
            //   pos[index]
            if self.merkle_tree.get_next_index() < index + 2 {
                old_leaves.push([0u8; 32]);
            } else {
                old_leaves.push(*leaf);
            }
            // Handle case that we nullify a leaf which has not been inserted yet.
            while self.merkle_tree.get_next_index() < index + 2 {
                self.merkle_tree.append(&[0u8; 32]).unwrap();
            }
            let proof = self.merkle_tree.get_proof_of_leaf(index, true).unwrap();
            merkle_proofs.push(proof.to_vec());
            path_indices.push(index as u32);
            self.input_queue_leaves.remove(0);
            let event = self
                .tx_events
                .iter()
                .find(|tx_event| tx_event.inputs.contains(leaf))
                .expect("No event for leaf found.");
            let index_bytes = index.to_be_bytes();
            let nullifier = Poseidon::hashv(&[leaf, &index_bytes, &event.tx_hash]).unwrap();
            tx_hashes.push(event.tx_hash);
            nullifiers.push(nullifier);
            self.merkle_tree.update(&nullifier, index).unwrap();
        }
        // local_leaves_hashchain is only used for a test assertion.
        let local_nullifier_hashchain = calculate_hash_chain(&nullifiers);
        assert_eq!(leaves_hashchain, local_nullifier_hashchain);
        let inputs = get_batch_update_inputs::<HEIGHT>(
            old_root,
            tx_hashes,
            leaves.iter().map(|(leaf, _)| *leaf).collect(),
            leaves_hashchain,
            old_leaves,
            merkle_proofs,
            path_indices,
            batch_size,
        );
        let client = Client::new();
        let circuit_inputs_new_root =
            bigint_to_be_bytes_array::<32>(&inputs.new_root.to_biguint().unwrap()).unwrap();
        let inputs = update_inputs_string(&inputs);
        let new_root = self.merkle_tree.root();

        let response_result = client
            .post(&format!("{}{}", SERVER_ADDRESS, PROVE_PATH))
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(inputs)
            .send()
            .await
            .expect("Failed to execute request.");
        assert_eq!(circuit_inputs_new_root, new_root);

        if response_result.status().is_success() {
            let body = response_result.text().await.unwrap();
            let proof_json = deserialize_gnark_proof_json(&body).unwrap();
            let (proof_a, proof_b, proof_c) = proof_from_json_struct(proof_json);
            let (proof_a, proof_b, proof_c) = compress_proof(&proof_a, &proof_b, &proof_c);
            return Ok((
                CompressedProof {
                    a: proof_a,
                    b: proof_b,
                    c: proof_c,
                },
                new_root,
            ));
        }
        Err(Error)
    }
}

pub struct CompressedProof {
    pub a: [u8; 32],
    pub b: [u8; 64],
    pub c: [u8; 32],
}
