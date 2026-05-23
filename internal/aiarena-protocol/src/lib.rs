//! Shared Rust compatibility types for the public AI Arena transport contract.

pub mod common;
pub mod gamemaster;
pub mod player;

pub use common::{
    DecodeError, Decoder, Encoder, ErrorObject, GameMetadata, JSONRPC_VERSION,
    MetadataCompatibilityError, Request, Response, Transport, major_version, match_response_id,
    metadata_compatible,
};
