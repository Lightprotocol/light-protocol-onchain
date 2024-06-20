/*
 * photon-indexer
 *
 * Solana indexer for general compression
 *
 * The version of the OpenAPI document: 0.26.0
 *
 * Generated by: https://openapi-generator.tech
 */

use crate::models;

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct TokenData {
    #[serde(rename = "amount")]
    pub amount: i32,
    /// A Solana public key represented as a base58 string.
    #[serde(rename = "delegate", skip_serializing_if = "Option::is_none")]
    pub delegate: Option<String>,
    #[serde(rename = "isNative", skip_serializing_if = "Option::is_none")]
    pub is_native: Option<i32>,
    /// A Solana public key represented as a base58 string.
    #[serde(rename = "mint")]
    pub mint: String,
    /// A Solana public key represented as a base58 string.
    #[serde(rename = "owner")]
    pub owner: String,
    #[serde(rename = "state")]
    pub state: models::AccountState,
}

impl TokenData {
    pub fn new(amount: i32, mint: String, owner: String, state: models::AccountState) -> TokenData {
        TokenData {
            amount,
            delegate: None,
            is_native: None,
            mint,
            owner,
            state,
        }
    }
}
