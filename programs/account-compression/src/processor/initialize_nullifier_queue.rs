use crate::{queue_from_bytes_zero_copy_init, AccessMetadata, NullifierQueue, RolloverMetadata};
use anchor_lang::{prelude::*, solana_program::pubkey::Pubkey};

pub fn process_initialize_nullifier_queue<'a, 'b, 'c: 'info, 'info>(
    nullifier_queue_account_info: AccountInfo<'info>,
    nullifier_queue_account_loader: &'a AccountLoader<'info, NullifierQueue>,
    index: u64,
    owner: Pubkey,
    delegate: Option<Pubkey>,
    associated_merkle_tree: Pubkey,
    capacity_indices: u16,
    capacity_values: u16,
    sequence_threshold: u64,
    rollover_threshold: Option<u64>,
    close_threshold: Option<u64>,
    network_fee: u64,
) -> Result<()> {
    {
        let mut nullifier_queue = nullifier_queue_account_loader.load_init()?;
        let rollover_meta_data = RolloverMetadata {
            index,
            rollover_threshold: rollover_threshold.unwrap_or_default(),
            close_threshold: close_threshold.unwrap_or(u64::MAX),
            rolledover_slot: u64::MAX,
            network_fee,
            rollover_fee: 0,
        };

        nullifier_queue.init(
            AccessMetadata {
                owner,
                delegate: delegate.unwrap_or_default(),
            },
            rollover_meta_data,
            associated_merkle_tree,
        );

        drop(nullifier_queue);
    }

    let nullifier_queue = nullifier_queue_account_info;
    let mut nullifier_queue = nullifier_queue.try_borrow_mut_data()?;
    let _ = unsafe {
        queue_from_bytes_zero_copy_init(
            &mut nullifier_queue,
            capacity_indices.into(),
            capacity_values.into(),
            sequence_threshold as usize,
        )
        .unwrap()
    };
    Ok(())
}
