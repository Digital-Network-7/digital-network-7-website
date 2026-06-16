//! Release store — the site's own origin of truth for pushed panel binaries.
//!
//! dn7.cn is no longer a thin proxy in front of GitHub: panel CI *pushes* each
//! freshly-built, signed binary here (`/api/panel/ingest`), and an operator
//! marks one version **stable**. The public download / version / installer
//! endpoints serve only that stable version, so a release reaches users only
//! after a human gates it (GitHub stays the always-latest "preview" channel the
//! panel can opt into separately).
//!
//! Layout under `DN7_DATA_DIR` (default `/var/dn7/website`):
//!   * `data/store.json`                       — this index (0600)
//!   * `data/binaries/dn7-panel-linux-<arch>-v<version>` — the signed binaries
//!
//! Binaries are stored verbatim, **including** the appended 64-byte Ed25519
//! signature, so a downloading panel re-verifies them against its embedded key.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

/// One stored architecture asset for a version.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArchAsset {
    /// Hex SHA-256 of the stored file (binary + appended signature).
    pub sha256: String,
    /// Size in bytes of the stored file.
    pub size: u64,
    /// File name under `binaries/` (also the public asset name).
    pub file: String,
}

/// One pushed version, with whatever architectures have been ingested so far.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VersionEntry {
    pub version: String,
    /// Unix seconds when this version was first ingested.
    #[serde(default)]
    pub uploaded_at: u64,
    /// arch (`x86_64` / `arm64`) -> asset.
    #[serde(default)]
    pub arches: BTreeMap<String, ArchAsset>,
}

/// The persisted release index.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Store {
    /// Operator-selected stable version (e.g. `1.4.7`). When unset/missing the
    /// newest ingested version is used as the effective stable.
    #[serde(default)]
    pub stable: Option<String>,
    /// All ingested versions (unordered on disk; sorted for display).
    #[serde(default)]
    pub versions: Vec<VersionEntry>,
}

/// Supported architecture tokens.
pub const ARCHES: [&str; 2] = ["x86_64", "arm64"];

/// Public/stored asset name for a (version, arch) pair — the same scheme the
/// panel's CI and updater use (`dn7-panel-linux-<arch>-v<version>`).
pub fn asset_name(version: &str, arch: &str) -> String {
    format!("dn7-panel-linux-{arch}-v{version}")
}

/// Base data dir (`DN7_DATA_DIR`, default `/var/dn7/website`).
pub fn data_root() -> PathBuf {
    PathBuf::from(std::env::var("DN7_DATA_DIR").unwrap_or_else(|_| "/var/dn7/website".to_string()))
}

fn data_dir() -> PathBuf {
    data_root().join("data")
}

pub fn binaries_dir() -> PathBuf {
    data_dir().join("binaries")
}

fn store_path() -> PathBuf {
    data_dir().join("store.json")
}

/// Parse a semver-ish `a.b.c` into a comparable tuple (missing parts = 0).
fn ver_key(v: &str) -> (u64, u64, u64) {
    let v = v.trim().trim_start_matches('v');
    let mut it = v.split('.');
    let a = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let b = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let c = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    (a, b, c)
}

impl Store {
    /// Load the index from disk, or a default empty store if absent/corrupt.
    pub fn load() -> Self {
        match std::fs::read(store_path()) {
            Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
            Err(_) => Store::default(),
        }
    }

    /// Persist the index atomically with 0600 perms.
    pub fn save(&self) -> Result<()> {
        let dir = data_dir();
        std::fs::create_dir_all(&dir).context("create data dir")?;
        let path = store_path();
        let tmp = dir.join(".store.json.tmp");
        let json = serde_json::to_vec_pretty(self)?;
        write_private(&tmp, &json)?;
        std::fs::rename(&tmp, &path).context("commit store.json")?;
        Ok(())
    }

    /// Find a version entry by version string.
    pub fn find(&self, version: &str) -> Option<&VersionEntry> {
        let version = version.trim_start_matches('v');
        self.versions.iter().find(|e| e.version == version)
    }

    /// Versions sorted newest-first (for the admin UI and `latest`).
    pub fn sorted(&self) -> Vec<&VersionEntry> {
        let mut v: Vec<&VersionEntry> = self.versions.iter().collect();
        v.sort_by_key(|e| std::cmp::Reverse(ver_key(&e.version)));
        v
    }

    /// The newest ingested version (by semver), if any.
    pub fn newest(&self) -> Option<&VersionEntry> {
        self.versions
            .iter()
            .max_by(|a, b| ver_key(&a.version).cmp(&ver_key(&b.version)))
    }

    /// The effective stable version: the operator-selected one when set and
    /// still present, otherwise the newest ingested version. `None` only when
    /// nothing has been ingested at all.
    pub fn effective_stable(&self) -> Option<&VersionEntry> {
        if let Some(sel) = self.stable.as_deref() {
            if let Some(e) = self.find(sel) {
                return Some(e);
            }
        }
        self.newest()
    }

    /// Record an ingested asset (merging into an existing version entry), then
    /// persist. Returns the resulting asset metadata.
    pub fn record(&mut self, version: &str, arch: &str, asset: ArchAsset) -> Result<()> {
        let version = version.trim_start_matches('v').to_string();
        let now = now_secs();
        match self.versions.iter_mut().find(|e| e.version == version) {
            Some(e) => {
                e.arches.insert(arch.to_string(), asset);
            }
            None => self.versions.push(VersionEntry {
                version,
                uploaded_at: now,
                arches: BTreeMap::from([(arch.to_string(), asset)]),
            }),
        }
        self.save()
    }
}

/// Write `bytes` to `path` with 0600 permissions (owner-only).
fn write_private(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)
        .with_context(|| format!("open {} for write", path.display()))?;
    f.write_all(bytes)?;
    f.flush()?;
    Ok(())
}

/// Persist a binary file (binary + appended signature) under `binaries/`,
/// returning the on-disk path. 0644 so it can be served; the bytes are public.
pub fn write_binary(file_name: &str, bytes: &[u8]) -> Result<PathBuf> {
    let dir = binaries_dir();
    std::fs::create_dir_all(&dir).context("create binaries dir")?;
    let path = dir.join(file_name);
    let tmp = dir.join(format!(".{file_name}.tmp"));
    std::fs::write(&tmp, bytes).context("write temp binary")?;
    std::fs::rename(&tmp, &path).context("commit binary")?;
    Ok(path)
}

/// Read a stored binary by file name.
pub fn read_binary(file_name: &str) -> Result<Vec<u8>> {
    // Defensive: the name is built server-side from validated version/arch, but
    // refuse any path separators just in case.
    if file_name.contains('/') || file_name.contains("..") {
        return Err(anyhow!("invalid asset name"));
    }
    std::fs::read(binaries_dir().join(file_name)).context("read stored binary")
}

/// Hex SHA-256 of `bytes`.
pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(bytes);
    let mut s = String::with_capacity(64);
    for b in digest {
        use std::fmt::Write;
        let _ = write!(s, "{b:02x}");
    }
    s
}

pub fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effective_stable_prefers_selected_then_newest() {
        let mut s = Store::default();
        assert!(s.effective_stable().is_none());
        s.versions.push(VersionEntry {
            version: "1.4.2".into(),
            uploaded_at: 1,
            arches: BTreeMap::new(),
        });
        s.versions.push(VersionEntry {
            version: "1.4.10".into(),
            uploaded_at: 2,
            arches: BTreeMap::new(),
        });
        // No selection -> newest by semver (1.4.10 > 1.4.2).
        assert_eq!(s.effective_stable().unwrap().version, "1.4.10");
        // Selection honoured when present.
        s.stable = Some("1.4.2".into());
        assert_eq!(s.effective_stable().unwrap().version, "1.4.2");
        // Stale selection falls back to newest.
        s.stable = Some("9.9.9".into());
        assert_eq!(s.effective_stable().unwrap().version, "1.4.10");
    }

    #[test]
    fn sha256_hex_is_lowercase_64() {
        let h = sha256_hex(b"abc");
        assert_eq!(h.len(), 64);
        assert_eq!(
            h,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
