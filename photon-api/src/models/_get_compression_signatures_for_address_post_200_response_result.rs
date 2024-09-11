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
pub struct GetCompressionSignaturesForAddressPost200ResponseResult {
    #[serde(rename = "context")]
    pub context: Box<models::Context>,
    #[serde(rename = "value")]
    pub value: Box<models::PaginatedSignatureInfoList>,
}

impl GetCompressionSignaturesForAddressPost200ResponseResult {
    pub fn new(
        context: models::Context,
        value: models::PaginatedSignatureInfoList,
    ) -> GetCompressionSignaturesForAddressPost200ResponseResult {
        GetCompressionSignaturesForAddressPost200ResponseResult {
            context: Box::new(context),
            value: Box::new(value),
        }
    }
}
