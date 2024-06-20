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
pub struct GetCompressedAccountPost200ResponseResult {
    #[serde(rename = "context")]
    pub context: Box<models::Context>,
    #[serde(rename = "value", skip_serializing_if = "Option::is_none")]
    pub value: Option<Box<models::Account>>,
}

impl GetCompressedAccountPost200ResponseResult {
    pub fn new(context: models::Context) -> GetCompressedAccountPost200ResponseResult {
        GetCompressedAccountPost200ResponseResult {
            context: Box::new(context),
            value: None,
        }
    }
}

