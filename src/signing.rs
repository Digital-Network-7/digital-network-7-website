//! Release-signature verification — the auth for the CI push endpoint.
//!
//! Every published DN7 Panel binary has a 64-byte Ed25519 signature appended to
//! its end (the panel's own CI signs it with the release private key, kept only
//! in Actions secrets). Deployed panels verify that signature against an
//! embedded public key before self-updating.
//!
//! This site reuses the EXACT SAME trust anchor: the push endpoint
//! (`/api/panel/ingest`) accepts a binary only if its appended signature
//! verifies against the key below. That makes the signature itself the
//! authentication — only the release-key holder (i.e. the panel's CI) can
//! produce a binary the site will store, so no separate shared token is needed.
//!
//! The key is the same raw 32-byte Ed25519 public key embedded in the panel
//! (`Panel/src/platform/signing/verify.rs`, fingerprint sha256[:16]
//! `8c8792efabded96d`). If the panel rotates its signing key, mirror the new
//! key here too.

use ed25519_dalek::{Signature, VerifyingKey};

/// Length of the Ed25519 signature appended to every published binary.
pub const SIG_LEN: usize = 64;

/// Trusted release-signing public key (raw Ed25519, 32 bytes) — identical to
/// the panel's embedded key so a binary the panel trusts is exactly the set of
/// binaries this site accepts.
const TRUSTED_KEYS: &[[u8; 32]] = &[[
    24, 96, 10, 98, 35, 106, 5, 224, 130, 245, 114, 38, 92, 39, 8, 102, 52, 58, 166, 33, 214, 2,
    215, 254, 39, 181, 85, 232, 69, 126, 217, 94,
]];

/// Verify a detached Ed25519 signature (`sig`, raw 64 bytes) over `data`
/// against any trusted key. Returns true only on a strict, valid signature.
pub fn verify(data: &[u8], sig: &[u8]) -> bool {
    let sig: [u8; 64] = match sig.try_into() {
        Ok(s) => s,
        Err(_) => return false,
    };
    let signature = Signature::from_bytes(&sig);
    TRUSTED_KEYS.iter().any(|k| {
        VerifyingKey::from_bytes(k)
            .map(|vk| vk.verify_strict(data, &signature).is_ok())
            .unwrap_or(false)
    })
}

/// Verify a full published file: the binary with its 64-byte signature
/// appended. Returns true iff the file is large enough and the trailing
/// signature verifies over the leading binary bytes. The file is left intact
/// (stored as-is, including the trailer, so the panel can re-verify on
/// download).
pub fn verify_appended(file: &[u8]) -> bool {
    if file.len() <= SIG_LEN {
        return false;
    }
    let split = file.len() - SIG_LEN;
    verify(&file[..split], &file[split..])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// "binary" + appended 64-byte signature produced by OpenSSL with the
    /// release key over the message "dn7-panel-signing-test" — the same fixture
    /// the panel uses, proving this site accepts exactly what the panel accepts.
    const SIG: [u8; 64] = [
        211, 133, 253, 20, 41, 65, 53, 133, 192, 5, 141, 183, 171, 14, 67, 104, 51, 101, 67, 19,
        119, 250, 153, 134, 141, 27, 153, 97, 137, 112, 38, 67, 214, 75, 236, 251, 138, 202, 255,
        32, 164, 4, 102, 36, 188, 21, 49, 159, 103, 216, 92, 170, 133, 159, 120, 126, 39, 228, 60,
        82, 73, 16, 62, 1,
    ];

    #[test]
    fn accepts_trusted_appended_signature() {
        let mut file = b"dn7-panel-signing-test".to_vec();
        file.extend_from_slice(&SIG);
        assert!(verify_appended(&file));
    }

    #[test]
    fn rejects_tampered_or_unsigned() {
        // Tampered leading byte.
        let mut bad = b"dn7-panel-signing-tesT".to_vec();
        bad.extend_from_slice(&SIG);
        assert!(!verify_appended(&bad));
        // Too small to even hold a signature.
        assert!(!verify_appended(&[0u8; 10]));
        assert!(!verify_appended(b"dn7-panel-signing-test"));
    }
}
