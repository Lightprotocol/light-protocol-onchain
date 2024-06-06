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
pub struct CompressedProof {
    #[serde(rename = "a")]
    pub a: std::path::PathBuf,
    #[serde(rename = "b")]
    pub b: std::path::PathBuf,
    #[serde(rename = "c")]
    pub c: std::path::PathBuf,
}

impl CompressedProof {
    pub fn new(a: std::path::PathBuf, b: std::path::PathBuf, c: std::path::PathBuf) -> CompressedProof {
        CompressedProof {
            a,
            b,
            c,
        }
    }
}

