/*
 * photon-indexer
 *
 * Solana indexer for general compression
 *
 * The version of the OpenAPI document: 0.45.0
 *
 * Generated by: https://openapi-generator.tech
 */

use crate::models;

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct SignatureInfoListWithError {
    #[serde(rename = "items")]
    pub items: Vec<models::SignatureInfoWithError>,
}

impl SignatureInfoListWithError {
    pub fn new(items: Vec<models::SignatureInfoWithError>) -> SignatureInfoListWithError {
        SignatureInfoListWithError { items }
    }
}