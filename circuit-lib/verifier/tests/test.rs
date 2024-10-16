#[cfg(test)]
mod test {
    use light_prover_client::batch_append_with_subtrees::{
        calculate_hash_chain, calculate_two_inputs_hash_chain,
    };
    use light_prover_client::gnark::helpers::{ProofType, ProverConfig};
    use light_prover_client::inclusion::merkle_tree_info::MerkleTreeInfo;
    use light_prover_client::init_merkle_tree::inclusion_merkle_tree_inputs;
    use light_prover_client::{
        gnark::{
            constants::{PROVE_PATH, SERVER_ADDRESS},
            helpers::{kill_prover, spawn_prover},
            inclusion_json_formatter::inclusion_inputs_string,
            proof_helpers::{compress_proof, deserialize_gnark_proof_json, proof_from_json_struct},
        },
        helpers::init_logger,
    };
    use light_verifier::{select_verifying_key, verify, CompressedProof};
    use reqwest::Client;
    use serial_test::serial;

    #[serial]
    #[tokio::test]
    async fn prove_inclusion() {
        init_logger();
        spawn_prover(
            true,
            ProverConfig {
                run_mode: None,
                circuits: vec![ProofType::Inclusion],
            },
        )
        .await;
        let client = Client::new();
        for number_of_compressed_accounts in &[1usize, 2, 3] {
            let big_int_inputs = inclusion_merkle_tree_inputs(MerkleTreeInfo::H26);

            let inputs = inclusion_inputs_string(*number_of_compressed_accounts);
            let response_result = client
                .post(&format!("{}{}", SERVER_ADDRESS, PROVE_PATH))
                .header("Content-Type", "text/plain; charset=utf-8")
                .body(inputs)
                .send()
                .await
                .expect("Failed to execute request.");
            assert!(response_result.status().is_success());
            let body = response_result.text().await.unwrap();
            let proof_json = deserialize_gnark_proof_json(&body).unwrap();
            let (proof_a, proof_b, proof_c) = proof_from_json_struct(proof_json);
            let (proof_a, proof_b, proof_c) = compress_proof(&proof_a, &proof_b, &proof_c);
            let mut roots = Vec::<[u8; 32]>::new();
            let mut leaves = Vec::<[u8; 32]>::new();

            for _ in 0..*number_of_compressed_accounts {
                roots.push(big_int_inputs.root.to_bytes_be().1.try_into().unwrap());
                leaves.push(big_int_inputs.leaf.to_bytes_be().1.try_into().unwrap());
            }
            let public_input_hash = calculate_two_inputs_hash_chain(&roots, &leaves);
            let vk = select_verifying_key(leaves.len(), 0).unwrap();
            verify::<1>(
                &[public_input_hash],
                &CompressedProof {
                    a: proof_a,
                    b: proof_b,
                    c: proof_c,
                },
                vk,
            )
            .unwrap();
        }
        kill_prover();
    }

    #[tokio::test]
    #[ignore]
    async fn prove_inclusion_full() {
        init_logger();
        spawn_prover(
            true,
            ProverConfig {
                run_mode: None,
                circuits: vec![ProofType::Inclusion],
            },
        )
        .await;
        let client = Client::new();
        for number_of_compressed_accounts in &[1usize, 2, 3, 4, 8] {
            let big_int_inputs = inclusion_merkle_tree_inputs(MerkleTreeInfo::H26);

            let inputs = inclusion_inputs_string(*number_of_compressed_accounts);
            let response_result = client
                .post(&format!("{}{}", SERVER_ADDRESS, PROVE_PATH))
                .header("Content-Type", "text/plain; charset=utf-8")
                .body(inputs)
                .send()
                .await
                .expect("Failed to execute request.");
            assert!(response_result.status().is_success());
            let body = response_result.text().await.unwrap();
            let proof_json = deserialize_gnark_proof_json(&body).unwrap();
            let (proof_a, proof_b, proof_c) = proof_from_json_struct(proof_json);
            let (proof_a, proof_b, proof_c) = compress_proof(&proof_a, &proof_b, &proof_c);
            let mut roots = Vec::<[u8; 32]>::new();
            let mut leaves = Vec::<[u8; 32]>::new();

            for _ in 0..*number_of_compressed_accounts {
                roots.push(big_int_inputs.root.to_bytes_be().1.try_into().unwrap());
                leaves.push(big_int_inputs.leaf.to_bytes_be().1.try_into().unwrap());
            }

            let roots_hash_chain = calculate_hash_chain(&roots);
            let leaves_hash_chain = calculate_hash_chain(&leaves);
            let public_input_hash = calculate_hash_chain(&[roots_hash_chain, leaves_hash_chain]);
            let vk = select_verifying_key(leaves.len(), 0).unwrap();
            verify::<1>(
                &[public_input_hash],
                &CompressedProof {
                    a: proof_a,
                    b: proof_b,
                    c: proof_c,
                },
                vk,
            )
            .unwrap();
        }
        kill_prover();
    }
}
