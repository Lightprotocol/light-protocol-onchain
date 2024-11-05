use std::ops::{Deref, DerefMut};

use anchor_lang::prelude::{AnchorDeserialize, AnchorSerialize, ProgramError, Pubkey, Result};
use light_hasher::{DataHasher, Discriminator, Hasher, Poseidon};
use light_utils::hash_to_bn254_field_size_be;

use crate::{
    account_info::LightAccountInfo,
    address::PackedNewAddressParams,
    error::LightSdkError,
    merkle_context::{pack_merkle_context, MerkleContext, PackedMerkleContext, RemainingAccounts},
};

pub trait LightAccounts<'a>: Sized {
    fn try_light_accounts(accounts: &'a [LightAccountInfo]) -> Result<Self>;
}

// TODO(vadorovsky): Implment `LightAccountLoader`.

/// A wrapper which abstracts away the UTXO model.
pub struct LightAccount<'info, T>
where
    T: AnchorDeserialize + AnchorSerialize + Clone + DataHasher + Default + Discriminator,
{
    account_state: T,
    account_info: &'info LightAccountInfo<'info>,
    input_hash: [u8; 32],
}

impl<'info, T> LightAccount<'info, T>
where
    T: AnchorDeserialize + AnchorSerialize + Clone + DataHasher + Default + Discriminator,
{
    pub fn from_light_account_info(account_info: &'info LightAccountInfo<'info>) -> Result<Self> {
        let account_state = if account_info.input.is_some() {
            if let Some(ref data) = account_info.data {
                T::try_from_slice(data.borrow().as_slice())?
            } else {
                return Err(LightSdkError::ExpectedData.into());
            }
        } else {
            T::default()
        };
        let input_hash = account_state
            .hash::<Poseidon>()
            .map_err(ProgramError::from)?;
        Ok(Self {
            account_state,
            account_info,
            input_hash,
        })
    }

    pub fn new_address_params(&self) -> Option<PackedNewAddressParams> {
        self.account_info.new_address
    }

    pub fn input_compressed_account(
        &self,
        program_id: &Pubkey,
    ) -> Result<Option<PackedCompressedAccountWithMerkleContext>> {
        match self.account_info.input.as_ref() {
            Some(input) => {
                let data = {
                    let discriminator = T::discriminator();
                    Some(CompressedAccountData {
                        discriminator,
                        data: Vec::new(),
                        data_hash: self.input_hash,
                    })
                };
                Ok(Some(PackedCompressedAccountWithMerkleContext {
                    compressed_account: CompressedAccount {
                        owner: *program_id,
                        lamports: input.lamports.unwrap_or(0),
                        address: input.address,
                        data,
                    },
                    merkle_context: input.merkle_context,
                    root_index: input.root_index,
                    read_only: false,
                }))
            }
            None => Ok(None),
        }
    }

    pub fn output_compressed_account(
        &self,
        program_id: &Pubkey,
    ) -> Result<Option<OutputCompressedAccountWithPackedContext>> {
        match self.account_info.output_merkle_tree_index {
            Some(merkle_tree_index) => {
                let data = {
                    let discriminator = T::discriminator();
                    let data_hash = self
                        .account_state
                        .hash::<Poseidon>()
                        .map_err(ProgramError::from)?;
                    Some(CompressedAccountData {
                        discriminator,
                        data: self.account_state.try_to_vec()?,
                        data_hash,
                    })
                };
                Ok(Some(OutputCompressedAccountWithPackedContext {
                    compressed_account: CompressedAccount {
                        owner: self.account_info.owner.unwrap_or(*program_id),
                        lamports: self.account_info.lamports.unwrap_or(0),
                        address: self.account_info.address,
                        data,
                    },
                    merkle_tree_index,
                }))
            }
            None => Ok(None),
        }
    }
}

impl<'a, T> Deref for LightAccount<'a, T>
where
    T: AnchorDeserialize + AnchorSerialize + Clone + DataHasher + Default + Discriminator,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.account_state
    }
}

impl<'a, T> DerefMut for LightAccount<'a, T>
where
    T: AnchorDeserialize + AnchorSerialize + Clone + DataHasher + Default + Discriminator,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.account_state
    }
}

#[derive(Debug, PartialEq, Default, Clone, AnchorDeserialize, AnchorSerialize)]
pub struct CompressedAccount {
    pub owner: Pubkey,
    pub lamports: u64,
    pub address: Option<[u8; 32]>,
    pub data: Option<CompressedAccountData>,
}

/// Hashing scheme:
/// H(owner || leaf_index || merkle_tree_pubkey || lamports || address || data.discriminator || data.data_hash)
impl CompressedAccount {
    pub fn hash_with_hashed_values<H: Hasher>(
        &self,
        &owner_hashed: &[u8; 32],
        &merkle_tree_hashed: &[u8; 32],
        leaf_index: &u32,
    ) -> Result<[u8; 32]> {
        let capacity = 3
            + std::cmp::min(self.lamports, 1) as usize
            + self.address.is_some() as usize
            + self.data.is_some() as usize * 2;
        let mut vec: Vec<&[u8]> = Vec::with_capacity(capacity);
        vec.push(owner_hashed.as_slice());

        // leaf index and merkle tree pubkey are used to make every compressed account hash unique
        let leaf_index = leaf_index.to_le_bytes();
        vec.push(leaf_index.as_slice());

        vec.push(merkle_tree_hashed.as_slice());

        // Lamports are only hashed if non-zero to safe CU
        // For safety we prefix the lamports with 1 in 1 byte.
        // Thus even if the discriminator has the same value as the lamports, the hash will be different.
        let mut lamports_bytes = [1, 0, 0, 0, 0, 0, 0, 0, 0];
        if self.lamports != 0 {
            lamports_bytes[1..].copy_from_slice(&self.lamports.to_le_bytes());
            vec.push(lamports_bytes.as_slice());
        }

        if self.address.is_some() {
            vec.push(self.address.as_ref().unwrap().as_slice());
        }

        let mut discriminator_bytes = [2, 0, 0, 0, 0, 0, 0, 0, 0];
        if let Some(data) = &self.data {
            discriminator_bytes[1..].copy_from_slice(&data.discriminator);
            vec.push(&discriminator_bytes);
            vec.push(&data.data_hash);
        }
        let hash = H::hashv(&vec).map_err(ProgramError::from)?;
        Ok(hash)
    }

    pub fn hash<H: Hasher>(
        &self,
        &merkle_tree_pubkey: &Pubkey,
        leaf_index: &u32,
    ) -> Result<[u8; 32]> {
        self.hash_with_hashed_values::<H>(
            &hash_to_bn254_field_size_be(&self.owner.to_bytes())
                .unwrap()
                .0,
            &hash_to_bn254_field_size_be(&merkle_tree_pubkey.to_bytes())
                .unwrap()
                .0,
            leaf_index,
        )
    }
}

#[derive(Debug, PartialEq, Default, Clone, AnchorDeserialize, AnchorSerialize)]
pub struct CompressedAccountData {
    pub discriminator: [u8; 8],
    pub data: Vec<u8>,
    pub data_hash: [u8; 32],
}

#[derive(Debug, PartialEq, Default, Clone, AnchorDeserialize, AnchorSerialize)]
pub struct CompressedAccountWithMerkleContext {
    pub compressed_account: CompressedAccount,
    pub merkle_context: MerkleContext,
}

impl CompressedAccountWithMerkleContext {
    pub fn new_init_account(
        owner: Pubkey,
        lamports: u64,
        address: Option<[u8; 32]>,
        merkle_context: MerkleContext,
    ) -> Self {
        Self {
            compressed_account: CompressedAccount {
                owner,
                lamports,
                address,
                data: None,
            },
            merkle_context,
        }
    }

    pub fn hash(&self) -> Result<[u8; 32]> {
        self.compressed_account.hash::<Poseidon>(
            &self.merkle_context.merkle_tree_pubkey,
            &self.merkle_context.leaf_index,
        )
    }
}

#[derive(Debug, PartialEq, Default, Clone, AnchorDeserialize, AnchorSerialize)]
pub struct PackedCompressedAccountWithMerkleContext {
    pub compressed_account: CompressedAccount,
    pub merkle_context: PackedMerkleContext,
    /// Index of root used in inclusion validity proof.
    pub root_index: u16,
    /// Placeholder to mark accounts read-only unimplemented set to false.
    pub read_only: bool,
}

#[derive(Debug, PartialEq, Default, Clone, AnchorDeserialize, AnchorSerialize)]
pub struct OutputCompressedAccountWithPackedContext {
    pub compressed_account: CompressedAccount,
    pub merkle_tree_index: u8,
}

/// Hashes a compressed account.
///
/// This function should be used for input accounts, where including only a
/// hash is sufficient.
pub fn hash_input_account<T>(account: &T) -> Result<CompressedAccountData>
where
    T: AnchorSerialize + DataHasher + Discriminator,
{
    let data_hash = account.hash::<Poseidon>().map_err(ProgramError::from)?;
    Ok(CompressedAccountData {
        discriminator: T::discriminator(),
        // Sending only data hash to the system program is sufficient.
        data: Vec::new(),
        data_hash,
    })
}

/// Serializes and hashes a compressed account.
///
/// This function should be used for output accounts, where data has to be
/// included for system-program to log in the ledger.
pub fn serialize_and_hash_output_account<T>(account: &T) -> Result<CompressedAccountData>
where
    T: AnchorSerialize + DataHasher + Discriminator,
{
    let data = account.try_to_vec()?;
    let data_hash = account.hash::<Poseidon>().map_err(ProgramError::from)?;
    Ok(CompressedAccountData {
        discriminator: T::discriminator(),
        data,
        data_hash,
    })
}

pub fn pack_compressed_accounts(
    compressed_accounts: &[CompressedAccountWithMerkleContext],
    root_indices: &[u16],
    remaining_accounts: &mut RemainingAccounts,
) -> Vec<PackedCompressedAccountWithMerkleContext> {
    compressed_accounts
        .iter()
        .zip(root_indices.iter())
        .map(|(x, root_index)| PackedCompressedAccountWithMerkleContext {
            compressed_account: x.compressed_account.clone(),
            merkle_context: pack_merkle_context(&x.merkle_context, remaining_accounts),
            root_index: *root_index,
            read_only: false,
        })
        .collect::<Vec<_>>()
}

pub fn pack_compressed_account(
    compressed_account: CompressedAccountWithMerkleContext,
    root_index: u16,
    remaining_accounts: &mut RemainingAccounts,
) -> PackedCompressedAccountWithMerkleContext {
    pack_compressed_accounts(&[compressed_account], &[root_index], remaining_accounts)[0].clone()
}
