//! Shared Rust compatibility types for the public AI Arena transport contract.

pub mod common;
pub mod gamemaster;
pub mod player;

pub use common::{
    DecodeError, Encoder, ErrorObject, GameMetadata, MetadataCompatibilityError, Request, Response,
    Transport, major_version, match_response_id, metadata_compatible,
};
