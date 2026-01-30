//! Hiding erasure-correcting codes (HECC) implementation based on the MCP whitepaper.
//!
//! # HECC Example
//! ```rust
//! use hecc::{hecc_encode, hecc_decode};
//!
//! let m = vec![1u8, 2, 3];
//! let r = vec![9u8, 8];
//! let shreds = hecc_encode(&m, &r, 7).unwrap();
//! let shards = vec![(shreds[0], 1), (shreds[2], 3), (shreds[4], 5), (shreds[5], 6), (shreds[6], 7)];
//! let (m2, r2) = hecc_decode(&shards, m.len(), r.len()).unwrap();
//! assert_eq!(m2, m);
//! assert_eq!(r2, r);
//! ```
//!
//! # Pipeline Example
//! ```rust
//! use hecc::{hecc_recover_bytes, hecc_shred_bytes, HeccParams};
//! use rand::{rngs::StdRng, SeedableRng};
//!
//! let params = HeccParams { k: 8, t: 4, n: 16 };
//! let mut rng = StdRng::seed_from_u64(1);
//! let payload = b"hello pipeline";
//! let shards = hecc_shred_bytes(params, payload, &mut rng).unwrap();
//! // Simulate loss: keep first K+T shards per block
//! let recovered = hecc_recover_bytes(params, &shards).unwrap();
//! assert_eq!(recovered, payload);
//! ```

mod gf256;
mod hecc_core;
mod hecc_pipeline;

pub use crate::hecc_core::{hecc_decode, hecc_encode, HeccError, HeccParams};
pub use crate::hecc_pipeline::{hecc_recover_bytes, hecc_shred_bytes, HeccPipelineError, HeccShard};
