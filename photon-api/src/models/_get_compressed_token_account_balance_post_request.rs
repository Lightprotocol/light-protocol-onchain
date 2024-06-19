/*
 * photon-indexer
 *
 * Solana indexer for general compression
 *
 * The version of the OpenAPI document: 0.25.0
 *
 * Generated by: https://openapi-generator.tech
 */

use crate::models;

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct GetCompressedTokenAccountBalancePostRequest {
    /// An ID to identify the request.
    #[serde(rename = "id")]
    pub id: Id,
    /// The version of the JSON-RPC protocol.
    #[serde(rename = "jsonrpc")]
    pub jsonrpc: Jsonrpc,
    /// The name of the method to invoke.
    #[serde(rename = "method")]
    pub method: Method,
    #[serde(rename = "params")]
    pub params: Box<models::GetCompressedAccountPostRequestParams>,
}

impl GetCompressedTokenAccountBalancePostRequest {
    pub fn new(
        id: Id,
        jsonrpc: Jsonrpc,
        method: Method,
        params: models::GetCompressedAccountPostRequestParams,
    ) -> GetCompressedTokenAccountBalancePostRequest {
        GetCompressedTokenAccountBalancePostRequest {
            id,
            jsonrpc,
            method,
            params: Box::new(params),
        }
    }
}
/// An ID to identify the request.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Id {
    #[serde(rename = "test-account")]
    TestAccount,
}

impl Default for Id {
    fn default() -> Id {
        Self::TestAccount
    }
}
/// The version of the JSON-RPC protocol.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Jsonrpc {
    #[serde(rename = "2.0")]
    Variant2Period0,
}

impl Default for Jsonrpc {
    fn default() -> Jsonrpc {
        Self::Variant2Period0
    }
}
/// The name of the method to invoke.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Method {
    #[serde(rename = "getCompressedTokenAccountBalance")]
    GetCompressedTokenAccountBalance,
}

impl Default for Method {
    fn default() -> Method {
        Self::GetCompressedTokenAccountBalance
    }
}
