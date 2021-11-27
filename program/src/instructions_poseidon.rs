use std::ops::Mul;
use std::ops::Add;
use std::ops::AddAssign;
use crate::poseidon_round_constants_split;
use solana_program::{
    msg,
    log::sol_log_compute_units,
};
use ark_ff::bytes::{FromBytes, ToBytes};
use ark_ff::BigInteger;

use ark_ed_on_bn254::Fq;
use arkworks_gadgets::poseidon::{PoseidonParameters, PoseidonError, Rounds,circom::CircomCRH, sbox::PoseidonSbox};
use arkworks_gadgets::utils;
use ark_crypto_primitives::{crh::{TwoToOneCRH, CRH}, Error};
use ark_std::{One, Zero};

//configuration for the poseidon hash to be compatible with circom bn254 with 2 inputs
#[derive(Default, Clone)]
pub struct PoseidonCircomRounds3;

impl Rounds for PoseidonCircomRounds3 {
    const FULL_ROUNDS: usize = 8;
    const PARTIAL_ROUNDS: usize = 57;
    const SBOX: PoseidonSbox = PoseidonSbox::Exponentiation(5);
    const WIDTH: usize = 3;
}

//instructions
pub fn permute_instruction_first(state: &mut Vec<Vec<u8>>, current_round: &mut usize, current_round_index: &mut usize, left_input: &[u8], right_input: &[u8]) -> Vec<Fq> {
    //parsing poseidon inputs to Fq elements and performing the first 4 full round permutations
    let rounds =  poseidon_round_constants_split::get_rounds_poseidon_circom_bn254_x5_3_split(0);
    let mds = poseidon_round_constants_split::get_mds_poseidon_circom_bn254_x5_3();
    let params = PoseidonParameters::<Fq>::new(rounds, mds);

    //parsing poseidon inputs to Fq elements
    let mut state_new =prepare_inputs(&params, &left_input, &right_input).unwrap();

    //performing the first 4 full round permutations
    state_new = permute_custom_split(&params, state_new, *current_round, 4).unwrap();

    *current_round += 4;
    //incrementing round index to fetch the right parameters next iteration
    *current_round_index += 1;

    //parsing state back into the account
    for (i, input_state) in state.iter_mut().enumerate() {
        <Fq as ToBytes>::write(&state_new[i], &mut input_state[..]);
    }
    state_new
}

pub fn permute_instruction_6(state: &mut Vec<Vec<u8>>, current_round: &mut usize, current_round_index: &mut usize){
    //6 permute poseidon instructions which should be inner instructions
    let rounds = poseidon_round_constants_split::get_rounds_poseidon_circom_bn254_x5_3_split(*current_round_index);
    let mds = poseidon_round_constants_split::get_mds_poseidon_circom_bn254_x5_3();
    let params = PoseidonParameters::<Fq>::new(rounds, mds);

    let mut state_new = Vec::new();
    for i in state.iter() {
        state_new.push(<Fq as FromBytes>::read(&i[..]).unwrap());
    }
    let state_new = permute_custom_split(&params, state_new, *current_round, 6).unwrap();

    *current_round += 6;
    //incrementing round index to fetch the right parameters next iteration
    *current_round_index += 1;

    //parsing state back into the account
    for (i, input_state) in state.iter_mut().enumerate() {
        <Fq as ToBytes>::write(&state_new[i], &mut input_state[..]);
    }
}

pub fn permute_instruction_3(state: &mut Vec<Vec<u8>>, current_round: &mut usize, current_round_index: &mut usize){
    //3 permute poseidon instructions which should be inner instructions to fill up the 65 rounds
    let rounds = poseidon_round_constants_split::get_rounds_poseidon_circom_bn254_x5_3_split(*current_round_index);
    let mds = poseidon_round_constants_split::get_mds_poseidon_circom_bn254_x5_3();
    let params = PoseidonParameters::<Fq>::new(rounds, mds);

    let mut state_new = Vec::new();
    for i in state.iter() {
        state_new.push(<Fq as FromBytes>::read(&i[..]).unwrap());
    }
    let state_new = permute_custom_split(&params, state_new, *current_round, 3).unwrap();

    *current_round += 3;
    //incrementing round index to fetch the right parameters next iteration
    *current_round_index += 1;

    //parsing state back into the account
    for (i, input_state) in state.iter_mut().enumerate() {
        <Fq as ToBytes>::write(&state_new[i], &mut input_state[..]);
    }
}

pub fn permute_instruction_last(state: &mut Vec<Vec<u8>>, current_round: &mut usize, current_round_index: &mut usize) {
    //4 permute poseidon instructions for the second half of full rounds at the end
    let rounds = poseidon_round_constants_split::get_rounds_poseidon_circom_bn254_x5_3_split(*current_round_index);
    let mds = poseidon_round_constants_split::get_mds_poseidon_circom_bn254_x5_3();
    let params = PoseidonParameters::<Fq>::new(rounds, mds);

    let mut state_new = Vec::new();
    for i in state.iter() {
        state_new.push(<Fq as FromBytes>::read(&i[..]).unwrap());
    }
    state_new = permute_custom_split(&params, state_new, *current_round, 4).unwrap();
    //msg!("Hash: {:?}", state_new[0]);

    //could be ommitted since the hash is computed
    *current_round += 4;
    *current_round_index += 1;

    //parsing state back into the account, the resulting hash is in state[0]
    for (i, input_state) in state.iter_mut().enumerate() {
        <Fq as ToBytes>::write(&state_new[i], &mut input_state[..]);
    }

}

//foundational functions for instructions
pub fn prepare_inputs(parameters: &PoseidonParameters::<Fq>, left_input: &[u8], right_input: &[u8]) -> Result<Vec<Fq>, Error>/*-> Result<Self::Output, Error> */{
    //modified from arkworks_gadgets

    const INPUT_SIZE_BITS: usize = ark_ff::biginteger::BigInteger256::NUM_LIMBS * 8 * PoseidonCircomRounds3::WIDTH * 8;
    const LEFT_INPUT_SIZE_BITS: usize = INPUT_SIZE_BITS / 2;
	const RIGHT_INPUT_SIZE_BITS: usize = INPUT_SIZE_BITS / 2;
    assert_eq!(left_input.len(), right_input.len());
    assert!(left_input.len() * 8 <= LEFT_INPUT_SIZE_BITS);
    let chained: Vec<_> = left_input
        .iter()
        .chain(right_input.iter())
        .copied()
        .collect();

    let f_inputs = utils::to_field_elements(&chained)?;
    if f_inputs.len() >= PoseidonCircomRounds3::WIDTH {
        panic!(
            "incorrect input length {:?} for width {:?} -- input bits {:?}",
            f_inputs.len(),
            PoseidonCircomRounds3::WIDTH,
            chained.len()
        );
    }

    let mut buffer = vec![Fq::zero()];
    for f in f_inputs {
        buffer.push(f);
    }
    Ok(buffer)

}

pub fn permute_custom_split(params: &PoseidonParameters<Fq>, mut state: Vec<Fq>, nr_start: usize, nr_iterations: usize ) -> Result<Vec<Fq>, PoseidonError> {
    //modified from arkworks_gadgets

    //let nr = P::FULL_ROUNDS + P::PARTIAL_ROUNDS;
    let nr_end = nr_start + nr_iterations;

    for r in nr_start..nr_end {
        state.iter_mut().enumerate().for_each(|(i, a)| {
            let c = params.round_keys[((r - nr_start) * PoseidonCircomRounds3::WIDTH + i)];
            a.add_assign(c);
        });

        let half_rounds = PoseidonCircomRounds3::FULL_ROUNDS / 2;
        if r < half_rounds || r >= (half_rounds + PoseidonCircomRounds3::PARTIAL_ROUNDS) {
            state
                .iter_mut()
                .try_for_each(|a| PoseidonCircomRounds3::SBOX.apply_sbox(*a).map(|f| *a = f))?;
        } else {
            state[0] = PoseidonCircomRounds3::SBOX.apply_sbox(state[0])?;
        }

        state = state
            .iter()
            .enumerate()
            .map(|(i, _)| {
                state.iter().enumerate().fold(Fq::zero(), |acc, (j, a)| {
                    let m = params.mds_matrix[i][j];
                    acc.add(m.mul(*a))
                })
            })
            .collect();
    }
    Ok(state)
}



#[cfg(test)]
mod tests {
    use super::*;
    use ark_std::{One};
    use arkworks_gadgets::utils;
    use ark_ff::{Field,PrimeField, BigInteger, FpParameters, Fp256};
    use arkworks_gadgets::utils::{
	get_mds_poseidon_circom_bn254_x5_3, get_rounds_poseidon_circom_bn254_x5_3, parse_vec,
    };
    use ark_std::{UniformRand, test_rng};
    use crate::state_merkle_tree::HashBytes;

    pub type PoseidonCircomCRH3 = CircomCRH<Fq, PoseidonCircomRounds3>;

    const INSTRUCTION_ORDER_POSEIDON_2_INPUTS : [u8; 12] = [0,1,1,1,1,1,1,1,1,1,2,3];

    //defining processor function for testing
    pub fn processor_poseidon(id: u8, account_struct: &mut HashBytes/*, data: &[u8]*/) {
        if id == 0 {
            permute_instruction_first(&mut account_struct.state,&mut account_struct.current_round, &mut account_struct.current_round_index, &account_struct.left, &account_struct.right);

        } else if id == 1{
            permute_instruction_6(&mut account_struct.state,&mut account_struct.current_round, &mut account_struct.current_round_index);

        } else if id == 2 {
            permute_instruction_3(&mut account_struct.state,&mut account_struct.current_round, &mut account_struct.current_round_index);

        } else if id == 3 {
            permute_instruction_last(&mut account_struct.state,&mut account_struct.current_round, &mut account_struct.current_round_index);

        }
    }


    #[test]
    fn offchain_test_poseidon_hash_instructions() {
        let rounds = get_rounds_poseidon_circom_bn254_x5_3::<Fq>();
	    let mds = get_mds_poseidon_circom_bn254_x5_3::<Fq>();
        let params = PoseidonParameters::<Fq>::new(rounds, mds);
        //perform the test 1000x
        for j in 0..1000 {
            //generating random test input
            let mut rng = test_rng();
            let left_input = Fp256::<ark_ed_on_bn254::FqParameters>::rand(&mut rng).into_repr().to_bytes_le();
            let right_input = Fp256::<ark_ed_on_bn254::FqParameters>::rand(&mut rng).into_repr().to_bytes_le();

            //generating reference poseidon hash with library to test against
        	let poseidon_res =
        		<PoseidonCircomCRH3 as TwoToOneCRH>::evaluate(&params, &left_input, &right_input)
        			.unwrap();

            //parsing reference hash to bytes
            let mut out_bytes = [0u8;32];
            <Fq as ToBytes>::write(&poseidon_res, &mut out_bytes[..]);

            //initing struct which similates onchain account for instructions
            let mut account_struct = HashBytes {
                is_initialized: true,
                state: vec![vec![0u8;32];3],
                current_round: 0,
                current_round_index: 0,
                left: left_input.to_vec(),
                right: right_input.to_vec(),
                currentLevelHash: vec![0u8;32],
                currentIndex: 0usize,
                currentLevel: 0usize,
                current_instruction_index: 0usize,
            };

            //executing poseidon instructions
            for i in INSTRUCTION_ORDER_POSEIDON_2_INPUTS {
                processor_poseidon(i, &mut account_struct,/* &vec![0u8;8], left_input.clone(), right_input.clone()].concat()*/);
            }

            assert_eq!(out_bytes.to_vec(), account_struct.state[0]);
        }
    }

    #[test]
    fn offchain_test_poseidon_hash_fails() {
        let rounds = get_rounds_poseidon_circom_bn254_x5_3::<Fq>();
	    let mds = get_mds_poseidon_circom_bn254_x5_3::<Fq>();
        let params = PoseidonParameters::<Fq>::new(rounds, mds);
        //perform the test 1000x
        for j in 0..1000 {
            //generating random test input
            let mut rng = test_rng();
            let left_input = Fp256::<ark_ed_on_bn254::FqParameters>::rand(&mut rng).into_repr().to_bytes_le();
            let right_input = Fp256::<ark_ed_on_bn254::FqParameters>::rand(&mut rng).into_repr().to_bytes_le();

            //generating reference poseidon hash with library to test against
        	let poseidon_res =
        		<PoseidonCircomCRH3 as TwoToOneCRH>::evaluate(&params, &left_input, &right_input)
        			.unwrap();

            //parsing reference hash to bytes
            let mut out_bytes = [0u8;32];
            <Fq as ToBytes>::write(&poseidon_res, &mut out_bytes[..]);

            //generating different random test input for second hash
            let right_input = Fp256::<ark_ed_on_bn254::FqParameters>::rand(&mut rng).into_repr().to_bytes_le();

            //initing struct which similates onchain account for instructions
            let mut account_struct = HashBytes {
                is_initialized: true,
                state: vec![vec![0u8;32];3],
                current_round: 0,
                current_round_index: 0,
                left: left_input.to_vec(),
                right: right_input.to_vec(),
                currentLevelHash: vec![0u8;32],
                currentIndex: 0usize,
                currentLevel: 0usize,
                current_instruction_index: 0usize,
            };


            //executing poseidon instructions
            for i in INSTRUCTION_ORDER_POSEIDON_2_INPUTS {
                processor_poseidon(i, &mut account_struct, /*&[vec![0u8;8], left_input.clone(), right_input.clone()].concat()*/);
            }

            assert!(out_bytes.to_vec() != account_struct.state[0]);
        }
    }
}
