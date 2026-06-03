//! Sign-in PIN, stored as an Argon2id hash (E10.6).
//!
//! The PIN never touches disk in the clear: [`set_pin`] runs it through Argon2id
//! with a random salt and writes the PHC string to `~/.config/mde/pin.hash` (mode
//! 0600). [`verify`] re-derives and compares in constant time. The hashing is a
//! pure function ([`hash_pin`]/[`verify_against`]) so the crypto is unit-tested
//! without touching the filesystem — and `mde lock` (E10.8) reuses [`verify`].
//!
//! Headless entry (`mde pin …`):
//!   --set <PIN>     enrol/replace the PIN
//!   --check         print whether a PIN is set (exit 0 set / 1 unset)
//!   --verify <PIN>  exit 0 if it matches, 1 otherwise
//!   --clear         remove the PIN

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use std::path::PathBuf;
use std::process::ExitCode;

/// `~/.config/mde/pin.hash` (honours `$XDG_CONFIG_HOME`, mirroring `state::config_path`).
pub fn pin_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("mde").join("pin.hash"))
}

/// Argon2id PHC string for `pin`. Pure (random salt aside) — the unit tests pin
/// the format and the verify round-trip without any I/O.
pub fn hash_pin(pin: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    Ok(Argon2::default()
        .hash_password(pin.as_bytes(), &salt)?
        .to_string())
}

/// Does `pin` match the stored PHC `hash`? False on any parse/verify failure.
pub fn verify_against(hash: &str, pin: &str) -> bool {
    PasswordHash::new(hash)
        .map(|parsed| {
            Argon2::default()
                .verify_password(pin.as_bytes(), &parsed)
                .is_ok()
        })
        .unwrap_or(false)
}

/// Is a PIN currently enrolled (the hash file exists and is non-empty)?
pub fn is_set() -> bool {
    pin_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false)
}

/// Enrol/replace the PIN: hash it and write `pin.hash` atomically at mode 0600.
pub fn set_pin(pin: &str) -> std::io::Result<()> {
    let Some(path) = pin_path() else {
        return Ok(());
    };
    let hash = hash_pin(pin)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("hash.tmp");
    std::fs::write(&tmp, &hash)?;
    // Owner-only — the hash should not be world-readable.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600))?;
    }
    std::fs::rename(&tmp, &path)
}

/// Verify `pin` against the stored hash (used by the lock screen, E10.8).
pub fn verify(pin: &str) -> bool {
    pin_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|stored| verify_against(stored.trim(), pin))
        .unwrap_or(false)
}

/// Remove the enrolled PIN, if any.
pub fn clear() -> std::io::Result<()> {
    if let Some(path) = pin_path() {
        if path.exists() {
            std::fs::remove_file(path)?;
        }
    }
    Ok(())
}

/// Headless entry point for `mde pin …` (see module docs).
pub fn run(args: &[String]) -> ExitCode {
    let val = |flag: &str| {
        args.iter()
            .position(|a| a == flag)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    if let Some(pin) = val("--set") {
        return match set_pin(&pin) {
            Ok(()) => {
                println!("PIN set.");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("mde pin: {e}");
                ExitCode::FAILURE
            }
        };
    }
    if let Some(pin) = val("--verify") {
        return if verify(&pin) {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        };
    }
    if args.iter().any(|a| a == "--clear") {
        let _ = clear();
        println!("PIN cleared.");
        return ExitCode::SUCCESS;
    }
    // Default / --check.
    if is_set() {
        println!("A PIN is set.");
        ExitCode::SUCCESS
    } else {
        println!("No PIN is set.");
        ExitCode::FAILURE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_argon2_phc_and_not_plaintext() {
        let h = hash_pin("1234").unwrap();
        assert!(
            h.starts_with("$argon2"),
            "expected a PHC argon2 string: {h}"
        );
        assert!(!h.contains("1234"), "the PIN must not appear in the hash");
    }

    #[test]
    fn verify_round_trips_and_rejects() {
        let h = hash_pin("8080").unwrap();
        assert!(verify_against(&h, "8080"));
        assert!(!verify_against(&h, "8081"));
        assert!(!verify_against("not-a-hash", "8080"));
    }

    #[test]
    fn distinct_salts_give_distinct_hashes() {
        // Same PIN, two enrolments → different stored strings (random salt).
        assert_ne!(hash_pin("0000").unwrap(), hash_pin("0000").unwrap());
    }
}
