use bytemuck::{Pod, Zeroable};

use crate::errors::MerkleTreeMetadataError;
use solana_program::msg;

#[cfg(feature = "anchor")]
use anchor_lang::{AnchorDeserialize, AnchorSerialize};
#[cfg(not(feature = "anchor"))]
use borsh::{BorshDeserialize as AnchorDeserialize, BorshSerialize as AnchorSerialize};

#[repr(C)]
#[derive(
    AnchorDeserialize, AnchorSerialize, Debug, PartialEq, Default, Pod, Zeroable, Clone, Copy,
)]
pub struct RolloverMetadata {
    /// Unique index.
    pub index: u64,
    /// This fee is used for rent for the next account.
    /// It accumulates in the account so that once the corresponding Merkle tree account is full it can be rolled over
    pub rollover_fee: u64,
    /// The threshold in percentage points when the account should be rolled over (95 corresponds to 95% filled).
    pub rollover_threshold: u64,
    /// Tip for maintaining the account.
    pub network_fee: u64,
    /// The slot when the account was rolled over, a rolled over account should not be written to.
    pub rolledover_slot: u64,
    /// If current slot is greater than rolledover_slot + close_threshold and
    /// the account is empty it can be closed. No 'close' functionality has been
    /// implemented yet.
    pub close_threshold: u64,
    /// Placeholder for bytes of additional accounts which are tied to the
    /// Merkle trees operation and need to be rolled over as well.
    pub additional_bytes: u64,
}

impl RolloverMetadata {
    pub fn new(
        index: u64,
        rollover_fee: u64,
        rollover_threshold: Option<u64>,
        network_fee: u64,
        close_threshold: Option<u64>,
        additional_bytes: Option<u64>,
    ) -> Self {
        Self {
            index,
            rollover_fee,
            rollover_threshold: rollover_threshold.unwrap_or(u64::MAX),
            network_fee,
            rolledover_slot: u64::MAX,
            close_threshold: close_threshold.unwrap_or(u64::MAX),
            additional_bytes: additional_bytes.unwrap_or_default(),
        }
    }

    pub fn rollover(&mut self) -> Result<(), MerkleTreeMetadataError> {
        if self.rollover_threshold == u64::MAX {
            return Err(MerkleTreeMetadataError::RolloverNotConfigured);
        }
        if self.rolledover_slot != u64::MAX {
            return Err(MerkleTreeMetadataError::MerkleTreeAlreadyRolledOver);
        }

        #[cfg(target_os = "solana")]
        {
            use solana_program::clock::Clock;
            use solana_program::sysvar::Sysvar;
            self.rolledover_slot = Clock::get().unwrap().slot;
        }
        #[cfg(not(target_os = "solana"))]
        {
            self.rolledover_slot = 1;
        }
        Ok(())
    }
}
pub fn check_rollover_fee_sufficient(
    rollover_fee: u64,
    queue_rent: u64,
    merkle_tree_rent: u64,
    rollover_threshold: u64,
    height: u32,
) -> Result<(), MerkleTreeMetadataError> {
    if rollover_fee != queue_rent + merkle_tree_rent
        && (rollover_fee * rollover_threshold * (2u64.pow(height))) / 100
            < queue_rent + merkle_tree_rent
    {
        msg!("rollover_fee: {}", rollover_fee);
        msg!("rollover_threshold: {}", rollover_threshold);
        msg!("height: {}", height);
        msg!("merkle_tree_rent: {}", merkle_tree_rent);
        msg!("queue_rent: {}", queue_rent);
        msg!(
            "((rollover_fee * rollover_threshold * (2u64.pow(height))) / 100): {} < {} rent",
            ((rollover_fee * rollover_threshold * (2u64.pow(height))) / 100),
            queue_rent + merkle_tree_rent
        );
        return Err(MerkleTreeMetadataError::InsufficientRolloverFee);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rollover_metadata() {
        let mut metadata = RolloverMetadata::new(0, 0, Some(95), 0, Some(100), Some(1));
        assert_eq!(metadata.rollover_threshold, 95);
        assert_eq!(metadata.close_threshold, 100);
        assert_eq!(metadata.rolledover_slot, u64::MAX);
        assert_eq!(metadata.additional_bytes, 1);

        metadata.rollover().unwrap();

        let mut metadata = RolloverMetadata::new(0, 0, None, 0, None, None);
        assert_eq!(metadata.rollover_threshold, u64::MAX);
        assert_eq!(metadata.close_threshold, u64::MAX);
        assert_eq!(metadata.additional_bytes, 0);

        assert_eq!(
            metadata.rollover(),
            Err(MerkleTreeMetadataError::RolloverNotConfigured.into())
        );
        let mut metadata = RolloverMetadata::new(0, 0, Some(95), 0, None, None);
        assert_eq!(metadata.close_threshold, u64::MAX);

        metadata.rollover().unwrap();
        let mut metadata = RolloverMetadata::new(0, 0, Some(95), 0, None, None);
        metadata.rolledover_slot = 0;
        assert_eq!(metadata.close_threshold, u64::MAX);

        assert_eq!(
            metadata.rollover(),
            Err(MerkleTreeMetadataError::MerkleTreeAlreadyRolledOver.into())
        );
    }
}