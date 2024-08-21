use std::{
    env,
    io::{self, prelude::*},
    process::{Command, Stdio},
    thread::spawn,
};

use ark_ff::PrimeField;
use num_bigint::BigUint;

use solana_program::keccak::hashv;
use thiserror::Error;

pub mod bigint;
pub mod fee;
pub mod offset;
pub mod prime;
pub mod rand;

#[derive(Debug, Error, PartialEq)]
pub enum UtilsError {
    #[error("Invalid input size, expected at most {0}")]
    InputTooLarge(usize),
    #[error("Invalid chunk size")]
    InvalidChunkSize,
    #[error("Invalid seeds")]
    InvalidSeeds,
    #[error("Invalid rollover thresold")]
    InvalidRolloverThreshold,
}

// NOTE(vadorovsky): Unfortunately, we need to do it by hand.
// `num_derive::ToPrimitive` doesn't support data-carrying enums.
impl From<UtilsError> for u32 {
    fn from(e: UtilsError) -> u32 {
        match e {
            UtilsError::InputTooLarge(_) => 12001,
            UtilsError::InvalidChunkSize => 12002,
            UtilsError::InvalidSeeds => 12003,
            UtilsError::InvalidRolloverThreshold => 12004,
        }
    }
}

impl From<UtilsError> for solana_program::program_error::ProgramError {
    fn from(e: UtilsError) -> Self {
        solana_program::program_error::ProgramError::Custom(e.into())
    }
}

pub fn is_smaller_than_bn254_field_size_be(bytes: &[u8; 32]) -> bool {
    let bigint = BigUint::from_bytes_be(bytes);
    bigint < ark_bn254::Fr::MODULUS.into()
}

pub fn hash_to_bn254_field_size_be(bytes: &[u8]) -> Option<([u8; 32], u8)> {
    let mut bump_seed = [u8::MAX];
    // Loops with decreasing bump seed to find a valid hash which is less than
    // bn254 Fr modulo field size.
    for _ in 0..u8::MAX {
        {
            let mut hashed_value: [u8; 32] = hashv(&[bytes, bump_seed.as_ref()]).to_bytes();
            // Truncates to 31 bytes so that value is less than bn254 Fr modulo
            // field size.
            hashed_value[0] = 0;
            if is_smaller_than_bn254_field_size_be(&hashed_value) {
                return Some((hashed_value, bump_seed[0]));
            }
        }
        bump_seed[0] -= 1;
    }
    None
}

/// Hashes the provided `bytes` with Keccak256 and ensures the result fits
/// in the BN254 prime field by repeatedly hashing the inputs with various
/// "bump seeds" and truncating the resulting hash to 31 bytes.
///
/// The attempted "bump seeds" are bytes from 255 to 0.
///
/// # Examples
///
/// ```
/// use light_utils::hashv_to_bn254_field_size_be;
///
/// hashv_to_bn254_field_size_be(&[b"foo", b"bar"]);
/// ```
pub fn hashv_to_bn254_field_size_be(bytes: &[&[u8]]) -> ([u8; 32], u8) {
    // Loops with decreasing bump seed to find a valid hash which is less than
    // bn254 Fr modulo field size.
    for bump_seed in (0..=u8::MAX).rev() {
        // NOTE(vadorovsky): This is, to my current knowledge, the least bad
        // solution for the problem of "adding one element to a slice".
        //
        // The `bytes` slice contains another slices/references to the actual
        // bytes. Therefore, adding these slices to the new vector doesn't
        // create a copy of the underlying bytes. What's more, adding the
        // slices to the `inputs` vector bints it to the lifefime of `bytes`.
        //
        // However, even though copies of the underlying bytes are not being
        // made, the copies of slices, as "fat pointers", are. Each slice takes
        // 16 bytes. Therefore, each iteration allocate aproximately
        // 16 * (n + 1) bytes. In the most cases, this loop should not iterate
        // more than once, so it's safe to assume that this function will
        // allocate 16 * (n + 1) all together, or twice this value in the worst
        // case. Due to the nature of Solana bump allocator, that memory is not
        // going to be freed after going out of this function's scope, if the
        // function is used on-chain. If that's problematic, the
        // `#[heap_neutral]` macro can be used as a workaround.
        let mut inputs = Vec::with_capacity(bytes.len() + 1);
        inputs.extend(bytes);
        let bump_seed = [bump_seed];
        inputs.push(bump_seed.as_slice());
        {
            let mut hashed_value: [u8; 32] = hashv(inputs.as_slice()).to_bytes();
            // Truncates to 31 bytes so that value is less than bn254 Fr modulo
            // field size.
            hashed_value[0] = 0;
            if is_smaller_than_bn254_field_size_be(&hashed_value) {
                return (hashed_value, bump_seed[0]);
            }
        }
        inputs.pop();
    }

    // PANICS: The probability of not finding any suitable bump seed in the
    // range 0-255 is extremely low, practically impossible to happen.
    // For the sake of not bloating this function with an annoying `Result` or
    // `Option` and being able to build macros which are using it, we are fine
    // with panicking here.
    unreachable!("Could not find the bump seed for provided inputs");
}

/// Applies `rustfmt` on the given string containing Rust code. The purpose of
/// this function is to be able to format autogenerated code (e.g. with `quote`
/// macro).
pub fn rustfmt(code: String) -> Result<Vec<u8>, anyhow::Error> {
    let mut cmd = match env::var_os("RUSTFMT") {
        Some(r) => Command::new(r),
        None => Command::new("rustfmt"),
    };

    let mut cmd = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let mut stdin = cmd.stdin.take().unwrap();
    let mut stdout = cmd.stdout.take().unwrap();

    let stdin_handle = spawn(move || {
        stdin.write_all(code.as_bytes()).unwrap();
    });

    let mut formatted_code = vec![];
    io::copy(&mut stdout, &mut formatted_code)?;

    let _ = cmd.wait();
    stdin_handle.join().unwrap();

    Ok(formatted_code)
}

#[cfg(test)]
mod tests {
    use num_bigint::ToBigUint;
    use solana_program::pubkey::Pubkey;

    use crate::bigint::bigint_to_be_bytes_array;

    use super::*;

    #[test]
    fn test_is_smaller_than_bn254_field_size_be() {
        let modulus: BigUint = ark_bn254::Fr::MODULUS.into();
        let modulus_bytes: [u8; 32] = bigint_to_be_bytes_array(&modulus).unwrap();
        assert!(!is_smaller_than_bn254_field_size_be(&modulus_bytes));

        let bigint = modulus.clone() - 1.to_biguint().unwrap();
        let bigint_bytes: [u8; 32] = bigint_to_be_bytes_array(&bigint).unwrap();
        assert!(is_smaller_than_bn254_field_size_be(&bigint_bytes));

        let bigint = modulus + 1.to_biguint().unwrap();
        let bigint_bytes: [u8; 32] = bigint_to_be_bytes_array(&bigint).unwrap();
        assert!(!is_smaller_than_bn254_field_size_be(&bigint_bytes));
    }

    #[test]
    fn test_hash_to_bn254_field_size_be() {
        for _ in 0..10_000 {
            let input_bytes = Pubkey::new_unique().to_bytes(); // Sample input
            let (hashed_value, bump) = hash_to_bn254_field_size_be(input_bytes.as_slice())
                .expect("Failed to find a hash within BN254 field size");
            assert_eq!(bump, 255, "Bump seed should be 0");
            assert!(
                is_smaller_than_bn254_field_size_be(&hashed_value),
                "Hashed value should be within BN254 field size"
            );
        }

        let max_input = [u8::MAX; 32];
        let (hashed_value, bump) = hash_to_bn254_field_size_be(max_input.as_slice())
            .expect("Failed to find a hash within BN254 field size");
        assert_eq!(bump, 255, "Bump seed should be 255");
        assert!(
            is_smaller_than_bn254_field_size_be(&hashed_value),
            "Hashed value should be within BN254 field size"
        );
    }

    #[test]
    fn test_hashv_to_bn254_field_size_be() {
        for _ in 0..10_000 {
            let input_bytes = [Pubkey::new_unique().to_bytes(); 4];
            let input_bytes = input_bytes.iter().map(|x| x.as_slice()).collect::<Vec<_>>();
            let (hashed_value, bump) = hashv_to_bn254_field_size_be(input_bytes.as_slice());
            assert_eq!(bump, 255, "Bump seed should be 0");
            assert!(
                is_smaller_than_bn254_field_size_be(&hashed_value),
                "Hashed value should be within BN254 field size"
            );
        }

        let max_input = [u8::MAX; 32];
        let (hashed_value, bump) = hash_to_bn254_field_size_be(max_input.as_slice())
            .expect("Failed to find a hash within BN254 field size");
        assert_eq!(bump, 255, "Bump seed should be 255");
        assert!(
            is_smaller_than_bn254_field_size_be(&hashed_value),
            "Hashed value should be within BN254 field size"
        );
    }

    #[test]
    fn test_rustfmt() {
        let unformatted_code = "use std::mem;

fn main() {        println!(\"{}\", mem::size_of::<u64>()); }
        "
        .to_string();
        let formatted_code = rustfmt(unformatted_code).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&formatted_code),
            "use std::mem;

fn main() {
    println!(\"{}\", mem::size_of::<u64>());
}
"
        );
    }
}
