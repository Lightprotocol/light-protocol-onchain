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
pub struct AccountList {
    #[serde(rename = "items")]
    pub items: Vec<models::Account>,
}

impl AccountList {
    pub fn new(items: Vec<models::Account>) -> AccountList {
        AccountList { items }
    }
}
