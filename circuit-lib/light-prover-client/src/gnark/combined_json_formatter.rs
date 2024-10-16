use num_bigint::BigInt;
use serde::Serialize;

use crate::batch_append_with_subtrees::calculate_hash_chain;
use crate::combined::merkle_combined_proof_inputs::CombinedProofInputs;
use crate::gnark::inclusion_json_formatter::BatchInclusionJsonStruct;
use crate::gnark::non_inclusion_json_formatter::BatchNonInclusionJsonStruct;
use crate::prove_utils::CircuitType;

use super::{
    helpers::{big_int_to_string, create_json_from_struct},
    inclusion_json_formatter::InclusionJsonStruct,
    non_inclusion_json_formatter::NonInclusionJsonStruct,
};

#[derive(Serialize, Debug)]
pub struct CombinedJsonStruct {
    #[serde(rename = "circuitType")]
    pub circuit_type: String,
    #[serde(rename = "stateTreeHeight")]
    pub state_tree_height: u32,
    #[serde(rename = "addressTreeHeight")]
    pub address_tree_height: u32,
    #[serde(rename = "publicInputHash")]
    pub public_input_hash: String,
    #[serde(rename(serialize = "inputCompressedAccounts"))]
    pub inclusion: Vec<InclusionJsonStruct>,

    #[serde(rename(serialize = "newAddresses"))]
    pub non_inclusion: Vec<NonInclusionJsonStruct>,
}

impl CombinedJsonStruct {
    fn new_with_public_inputs(number_of_utxos: usize) -> Self {
        let (inclusion, inclusion_public_input_hash) =
            BatchInclusionJsonStruct::new_with_public_inputs(number_of_utxos);
        let (non_inclusion, non_inclusion_public_input_hash) =
            BatchNonInclusionJsonStruct::new_with_public_inputs(number_of_utxos);

        let public_inputs_hash =
            calculate_hash_chain(&[inclusion_public_input_hash, non_inclusion_public_input_hash]);

        Self {
            circuit_type: CircuitType::Combined.to_string(),
            state_tree_height: 26,
            address_tree_height: 40,
            public_input_hash: big_int_to_string(&BigInt::from_bytes_be(
                num_bigint::Sign::Plus,
                public_inputs_hash.as_slice(),
            )),
            inclusion: inclusion.inputs,
            non_inclusion: non_inclusion.inputs,
        }
    }

    pub fn from_combined_inputs(inputs: &CombinedProofInputs) -> Self {
        let inclusion_parameters =
            BatchInclusionJsonStruct::from_inclusion_proof_inputs(&inputs.inclusion_parameters);
        let non_inclusion_parameters = BatchNonInclusionJsonStruct::from_non_inclusion_proof_inputs(
            &inputs.non_inclusion_parameters,
        );

        Self {
            circuit_type: CircuitType::Combined.to_string(),
            state_tree_height: 26,
            address_tree_height: 40,
            public_input_hash: big_int_to_string(&inputs.public_input_hash),
            inclusion: inclusion_parameters.inputs,
            non_inclusion: non_inclusion_parameters.inputs,
        }
    }

    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        create_json_from_struct(&self)
    }
}

pub fn combined_inputs_string(number_of_utxos: usize) -> String {
    let json_struct = CombinedJsonStruct::new_with_public_inputs(number_of_utxos);
    json_struct.to_string()
}
