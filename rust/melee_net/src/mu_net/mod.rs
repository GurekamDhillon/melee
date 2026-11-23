//! mu_net: Melee Unlocked's peer-to-peer gameplay protocol (version 1, revision 2), the wire of his launcher's
//! "P2P Direct". Our own implementation, written from his protocol document; the spec in our words, every rule's
//! source and the milestones are in docs/rust-port/notes/p2p-direct-plan.md.
//!
//! - `blake2b`, `crypto`: the primitives (BLAKE2b and HChaCha20 here, Ed25519 / X25519 / ChaCha20-Poly1305 from ring);
//! - `wire`: the message codecs; `desc`: the session descriptor and its digest; `sec`: HELLO, keys, packets.

pub mod blake2b;
pub mod conn;
pub mod crypto;
pub mod desc;
pub mod input;
pub mod link;
pub mod sec;
pub mod wire;
