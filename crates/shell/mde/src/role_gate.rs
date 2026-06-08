//! E1.2 — deployment-role gating of the desktop subcommands.
//!
//! A box pinned (via `mde setup --profile`, E1.1) to a non-Workstation role —
//! Lighthouse or Server — is headless: its GUI surfaces make no sense and there
//! is no Wayland session to host them. The dispatcher refuses those
//! subcommands with a clear "not available on this role" rather than launching
//! a window that can't paint.
//!
//! Three rules, all deliberate (§6 recommended-path choice, E1.2):
//!   * **Core CLI is never gated** — `panel`/`menu`/`files`/`net-flyout`/
//!     `filedialog` (and everything not in [`DESKTOP_ONLY`]) run on every role.
//!   * **Role/setup tools are never gated** — `mde setup --profile/--show`
//!     (the role pin + *upgrade* path) and the headless component install must
//!     work on every role, or a Lighthouse could never be upgraded. Only the
//!     *desktop* setup flows (the Win10 OOBE / GUI component picker) are gated,
//!     via [`check_desktop_setup`].
//!   * **Unpinned allows; malformed fails closed** — a box with no `role.toml`
//!     is pre-`mde setup` (or a dev tree) and runs everything; a *corrupt*
//!     `role.toml` refuses the desktop surfaces (fail closed), never assuming a
//!     Workstation default.

/// Pure desktop-UI subcommands — refused on a pinned non-Workstation role.
/// Deliberately excludes the core CLI and every setup/role tool (see the module
/// docs): adding `setup` here would break `mde setup --profile` upgrades.
const DESKTOP_ONLY: &[&str] = &["settings", "action-center", "security", "birthright"];

/// The shared role check: `Ok` when the desktop is permitted (Workstation, or
/// no role pinned yet), `Err(reason)` when a pinned non-Workstation role can't
/// host it or the pin is corrupt (fail closed).
fn role_allows_desktop() -> Result<(), String> {
    classify(mde_role::load())
}

/// Pure allow/refuse policy over a `role.toml` load result — split out so the
/// security-critical refusal logic is unit-tested without a `/var/lib/mde`
/// dependency.
fn classify(loaded: Result<mde_role::Role, mde_role::LoadError>) -> Result<(), String> {
    match loaded {
        Ok(mde_role::Role::Workstation) => Ok(()),
        Ok(role) => Err(format!(
            "not available on the {role} role (rank {})",
            role.rank()
        )),
        // No role.toml yet: pre-`mde setup`, or a dev checkout. Allow — the
        // role-gated RPM only ships desktop surfaces under the Workstation
        // subpackage, so a real headless box won't have them present anyway.
        Err(mde_role::LoadError::NotPinned) => Ok(()),
        // Corrupt pin: fail closed (refuse the desktop), never default to
        // Workstation.
        Err(e) => Err(format!("{e}; refusing the desktop surface (fail closed)")),
    }
}

/// Gate a subcommand by the pinned role. `Ok` to dispatch; `Err(message)` to
/// refuse (the caller prints it and exits non-zero). Subcommands outside
/// [`DESKTOP_ONLY`] always pass.
pub fn check(cmd: &str) -> Result<(), String> {
    if !DESKTOP_ONLY.contains(&cmd) {
        return Ok(());
    }
    role_allows_desktop().map_err(|why| format!("`mde {cmd}` is a desktop surface — {why}"))
}

/// Gate the *desktop* `mde setup` flows (the Win10 OOBE `--era=win10` and the
/// `--gui` component picker) by the pinned role. The headless `--tui` install
/// and the `--profile/--show` role CLI are NOT gated through here.
pub fn check_desktop_setup() -> Result<(), String> {
    role_allows_desktop().map_err(|why| format!("the desktop OOBE / GUI installer is {why}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_subcommands_are_never_in_the_desktop_set() {
        for core in [
            "panel",
            "menu",
            "files",
            "net-flyout",
            "filedialog",
            "setup",
            "install",
        ] {
            assert!(
                !DESKTOP_ONLY.contains(&core),
                "{core} must not be role-gated"
            );
        }
    }

    #[test]
    fn desktop_set_is_the_pure_ui_surfaces() {
        for ui in ["settings", "action-center", "security"] {
            assert!(DESKTOP_ONLY.contains(&ui), "{ui} should be role-gated");
        }
    }

    #[test]
    fn check_passes_core_regardless_of_role() {
        // A core subcommand short-circuits before the role is even read, so it
        // is always Ok (no /var/lib/mde dependency in the test environment).
        assert!(check("panel").is_ok());
        assert!(check("files").is_ok());
        assert!(check("setup").is_ok());
    }

    #[test]
    fn classify_allows_workstation_and_unpinned_refuses_the_rest() {
        use mde_role::{LoadError, Role};
        // Workstation + an unpinned (pre-setup / dev) box → desktop allowed.
        assert!(classify(Ok(Role::Workstation)).is_ok());
        assert!(classify(Err(LoadError::NotPinned)).is_ok());
        // A pinned non-Workstation role → refused.
        assert!(classify(Ok(Role::Server)).is_err());
        assert!(classify(Ok(Role::Lighthouse)).is_err());
        // A corrupt pin → fail closed (refused), never a Workstation default.
        assert!(classify(Err(LoadError::Malformed("bad".into()))).is_err());
    }
}
