use light_bloom_filter::BloomFilterError;
use light_bounded_vec::BoundedVecError;
use light_hasher::HasherError;
use light_merkle_tree_metadata::errors::MerkleTreeMetadataError;
use light_utils::UtilsError;
use light_verifier::VerifierError;
use solana_program::program_error::ProgramError;
use thiserror::Error;

use crate::zero_copy::ZeroCopyError;

#[derive(Debug, Error, PartialEq)]
pub enum BatchedMerkleTreeError {
    #[error("Batch is not ready to be inserted")]
    BatchNotReady,
    #[error("Batch is already inserted")]
    BatchAlreadyInserted,
    #[error("Batch insert failed")]
    BatchInsertFailed,
    #[error("Leaf index not in batch.")]
    LeafIndexNotInBatch,
    #[error("Invalid network fee.")]
    InvalidNetworkFee,
    #[error("Batch size not divisible by ZKP batch size.")]
    BatchSizeNotDivisibleByZkpBatchSize,
    #[error("Account owned by wrong program.")]
    AccountOwnedByWrongProgram,
    #[error("Account not mutable.")]
    AccountNotMutable,
    #[error("Inclusion proof by index failed.")]
    InclusionProofByIndexFailed,
    #[error("Hasher error: {0}")]
    Hasher(#[from] HasherError),
    #[error("Utils error {0}")]
    Utils(#[from] UtilsError),
    #[error("Bounded vector error: {0}")]
    BoundedVec(#[from] BoundedVecError),
    #[error("Zero copy error {0}")]
    ZeroCopy(#[from] ZeroCopyError),
    #[error("Merkle tree metadata error {0}")]
    MerkleTreeMetadata(#[from] MerkleTreeMetadataError),
    #[error("Bloom filter error {0}")]
    BloomFilter(#[from] BloomFilterError),
    #[error("Program error {0}")]
    ProgramError(#[from] ProgramError),
    #[error("Verifier error {0}")]
    VerifierErrorError(#[from] VerifierError),
}

#[cfg(feature = "solana")]
impl From<BatchedMerkleTreeError> for u32 {
    fn from(e: BatchedMerkleTreeError) -> u32 {
        match e {
            BatchedMerkleTreeError::BatchNotReady => 14301,
            BatchedMerkleTreeError::BatchAlreadyInserted => 14302,
            BatchedMerkleTreeError::BatchInsertFailed => 14303,
            BatchedMerkleTreeError::LeafIndexNotInBatch => 14304,
            BatchedMerkleTreeError::InvalidNetworkFee => 14305,
            BatchedMerkleTreeError::BatchSizeNotDivisibleByZkpBatchSize => 14306,
            BatchedMerkleTreeError::AccountOwnedByWrongProgram => 14307,
            BatchedMerkleTreeError::AccountNotMutable => 14308,
            BatchedMerkleTreeError::InclusionProofByIndexFailed => 14309,
            BatchedMerkleTreeError::Hasher(e) => e.into(),
            BatchedMerkleTreeError::BoundedVec(e) => e.into(),
            BatchedMerkleTreeError::ZeroCopy(e) => e.into(),
            BatchedMerkleTreeError::MerkleTreeMetadata(e) => e.into(),
            BatchedMerkleTreeError::BloomFilter(e) => e.into(),
            BatchedMerkleTreeError::VerifierErrorError(e) => e.into(),
            BatchedMerkleTreeError::Utils(e) => e.into(),
            BatchedMerkleTreeError::ProgramError(e) => u32::try_from(u64::from(e)).unwrap(),
        }
    }
}

#[cfg(feature = "solana")]
impl From<BatchedMerkleTreeError> for solana_program::program_error::ProgramError {
    fn from(e: BatchedMerkleTreeError) -> Self {
        solana_program::program_error::ProgramError::Custom(e.into())
    }
}