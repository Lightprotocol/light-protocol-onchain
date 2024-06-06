/*
 * photon-indexer
 *
 * Solana indexer for general compression
 *
 * The version of the OpenAPI document: 0.19.0
 *
 * Generated by: https://openapi-generator.tech
 */

use crate::models;

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Account {
    /// A Solana public key represented as a base58 string.
    #[serde(rename = "address", skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(rename = "data", skip_serializing_if = "Option::is_none")]
    pub data: Option<Box<models::AccountData>>,
    /// A 32-byte hash represented as a base58 string.
    #[serde(rename = "hash")]
    pub hash: String,
    #[serde(rename = "lamports")]
    pub lamports: i32,
    #[serde(rename = "leafIndex")]
    pub leaf_index: i32,
    /// A Solana public key represented as a base58 string.
    #[serde(rename = "owner")]
    pub owner: String,
    #[serde(rename = "seq", skip_serializing_if = "Option::is_none")]
    pub seq: Option<i32>,
    #[serde(rename = "slotCreated")]
    pub slot_created: i32,
    /// A Solana public key represented as a base58 string.
    #[serde(rename = "tree")]
    pub tree: String,
}

impl Account {
    pub fn new(
        hash: String,
        lamports: i32,
        leaf_index: i32,
        owner: String,
        slot_created: i32,
        tree: String,
    ) -> Account {
        Account {
            address: None,
            data: None,
            hash,
            lamports,
            leaf_index,
            owner,
            seq: None,
            slot_created,
            tree,
        }
    }
}
