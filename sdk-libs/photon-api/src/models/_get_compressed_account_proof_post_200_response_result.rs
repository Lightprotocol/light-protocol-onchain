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
pub struct GetCompressedAccountProofPost200ResponseResult {
    #[serde(rename = "context")]
    pub context: Box<models::Context>,
    #[serde(rename = "value")]
    pub value: Box<models::MerkleProofWithContext>,
}

impl GetCompressedAccountProofPost200ResponseResult {
    pub fn new(
        context: models::Context,
        value: models::MerkleProofWithContext,
    ) -> GetCompressedAccountProofPost200ResponseResult {
        GetCompressedAccountProofPost200ResponseResult {
            context: Box::new(context),
            value: Box::new(value),
        }
    }
}