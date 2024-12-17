use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum MerkleTreeMetadataError {
    #[error("Merkle tree and queue are not associated.")]
    MerkleTreeAndQueueNotAssociated,
    #[error("Rollover not configured.")]
    RolloverNotConfigured,
    #[error("Merkle tree already rolled over.")]
    MerkleTreeAlreadyRolledOver,
    #[error("Invalid queue type.")]
    InvalidQueueType,
    #[error("Insufficient rollover fee.")]
    InsufficientRolloverFee,
    #[error("Merkle tree not ready for rollover.")]
    NotReadyForRollover,
    #[error("Invalid tree type.")]
    InvalidTreeType,
}

#[cfg(feature = "solana")]
impl From<MerkleTreeMetadataError> for u32 {
    fn from(e: MerkleTreeMetadataError) -> u32 {
        match e {
            MerkleTreeMetadataError::MerkleTreeAndQueueNotAssociated => 14001,
            MerkleTreeMetadataError::RolloverNotConfigured => 14002,
            MerkleTreeMetadataError::MerkleTreeAlreadyRolledOver => 14003,
            MerkleTreeMetadataError::InvalidQueueType => 14004,
            MerkleTreeMetadataError::InsufficientRolloverFee => 14005,
            MerkleTreeMetadataError::NotReadyForRollover => 14006,
            MerkleTreeMetadataError::InvalidTreeType => 14007,
        }
    }
}

#[cfg(feature = "solana")]
impl From<MerkleTreeMetadataError> for solana_program::program_error::ProgramError {
    fn from(e: MerkleTreeMetadataError) -> Self {
        solana_program::program_error::ProgramError::Custom(e.into())
    }
}
