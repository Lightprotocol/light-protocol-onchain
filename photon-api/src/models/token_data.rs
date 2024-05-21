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
pub struct TokenData {
    #[serde(rename = "amount")]
    pub amount: i64,
    /// A Solana public key represented as a base58 string.
    #[serde(rename = "delegate", skip_serializing_if = "Option::is_none")]
    pub delegate: Option<String>,
    #[serde(rename = "delegatedAmount")]
    pub delegated_amount: i64,
    #[serde(rename = "isNative", skip_serializing_if = "Option::is_none")]
    pub is_native: Option<i64>,
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
    pub fn new(
        amount: i64,
        delegated_amount: i64,
        mint: String,
        owner: String,
        state: models::AccountState,
    ) -> TokenData {
        TokenData {
            amount,
            delegate: None,
            delegated_amount,
            is_native: None,
            mint,
            owner,
            state,
        }
    }
}
