//! KDC2-2 crypto trait surface — RSA-2048 pairing handshake +
//! AES-256-GCM session.
//!
//! Implementations land in KDC2-2.4. This file ships the **trait
//! shapes only** so the wire / discovery / plugins modules can
//! depend on them without forcing an early crypto-lib choice
//! (`ring` vs. `rust-crypto`; the v2.1 KDC2 lock keeps that open
//! until KDC2-2.4 explicitly surveys it).
//!
//! ## KeyStore is the seam for future post-quantum
//!
//! v2.1 explicitly omits post-quantum crypto per the KDC2 lock,
//! but the `KeyStore` trait below is where a future PQ adapter
//! will plug in — implementations expose key material as opaque
//! handles so a PQ algorithm swap doesn't touch wire/discovery/
//! plugins.

use std::fmt;

/// Opaque identifier for a key — used by the wire layer to
/// reference an active session key without exposing bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyHandle(pub u64);

impl fmt::Display for KeyHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "key#{:016x}", self.0)
    }
}

/// Errors any crypto operation may surface to the wire layer.
/// Stable variants here let the audit chain log a `family` token
/// (e.g. `"signature_invalid"`) without owning a giant flat enum.
#[derive(Debug)]
pub enum CryptoError {
    /// Pairing handshake signature failed to verify.
    SignatureInvalid,
    /// Session key not in the `KeyStore` (peer is no longer paired,
    /// or daemon was restarted without persisting the store).
    UnknownKey(KeyHandle),
    /// Encrypted body failed AEAD authentication — tampered or
    /// wrong key.
    AeadAuthFailed,
    /// Caller passed a key of the wrong algorithm (e.g. an AES key
    /// where an RSA key was expected).
    WrongAlgorithm,
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CryptoError::SignatureInvalid => write!(f, "signature_invalid"),
            CryptoError::UnknownKey(k) => write!(f, "unknown_key({k})"),
            CryptoError::AeadAuthFailed => write!(f, "aead_auth_failed"),
            CryptoError::WrongAlgorithm => write!(f, "wrong_algorithm"),
        }
    }
}

impl std::error::Error for CryptoError {}

/// Store for active session + identity keys. Implementations live
/// in `mde-kdc` (host integration); this crate uses the trait at
/// the wire layer's encrypt/decrypt boundary.
///
/// Object-safe so `mde-kdc` can hand a `Box<dyn KeyStore>` to the
/// wire decoder.
pub trait KeyStore: Send + Sync {
    /// Look up the session key bytes for a given handle.
    /// Implementations should clear the returned bytes on drop —
    /// callers MUST treat the slice as ephemeral and avoid copying
    /// it.
    ///
    /// Returns `None` when the handle is unknown (peer is not
    /// currently paired, or the key was rotated since the handle
    /// was issued).
    fn session_key(&self, handle: KeyHandle) -> Option<Vec<u8>>;

    /// Register a new session key after a successful pairing
    /// handshake. Returns the handle the wire layer uses going
    /// forward.
    fn install_session_key(&self, raw_key: &[u8]) -> KeyHandle;

    /// Forget a session key (peer unpaired, key rotation, etc.).
    /// Idempotent — calling with an unknown handle is a no-op.
    fn forget(&self, handle: KeyHandle);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_handle_display_is_stable_hex() {
        // The audit chain logs `key#<hex>` references; if the
        // formatting drifts the audit reader's regex breaks.
        let h = KeyHandle(0x1234);
        let s = format!("{h}");
        assert_eq!(s, "key#0000000000001234");
    }

    #[test]
    fn crypto_error_display_is_machine_token() {
        // Audit-log entries grep on the Display output. Each
        // variant must produce a stable single-token string.
        assert_eq!(format!("{}", CryptoError::SignatureInvalid), "signature_invalid");
        assert_eq!(format!("{}", CryptoError::AeadAuthFailed), "aead_auth_failed");
        assert_eq!(format!("{}", CryptoError::WrongAlgorithm), "wrong_algorithm");
        let s = format!("{}", CryptoError::UnknownKey(KeyHandle(1)));
        assert!(s.starts_with("unknown_key("));
    }

    #[test]
    fn key_handle_is_copy_and_hash() {
        // Used as a HashMap key in the wire layer's session
        // dispatch table — Copy + Hash + Eq must all be present.
        use std::collections::HashSet;
        let h = KeyHandle(7);
        let _copied = h; // doesn't move
        let mut set = HashSet::new();
        set.insert(h);
        assert!(set.contains(&KeyHandle(7)));
    }
}
