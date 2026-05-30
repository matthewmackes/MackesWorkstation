# MDE-Retro installer spec

Two front-ends, one install brain. `mde setup` **auto-detects**: no
`$WAYLAND_DISPLAY` (headless Fedora Server console) → **TUI**; inside a session →
the **GUI** Setup screen. Overrides: `--tui`, `--gui`, `--dry-run`.

## TUI (primary path — Fedora 44 Server, headless)
- **Look:** Windows 2000 text-mode Setup — full-screen blue, white/gray text,
  centered panel, bottom key-hint bar (`ENTER=Continue  F3=Exit`). ratatui +
  crossterm.
- **Privilege:** must run as **root** (`sudo mde setup`); installs with dnf and
  configures systemd directly (no polkit on a server). Non-root → error screen.
- **Delivery:** the `mde` binary comes from the **RPM**; `mde setup` is the
  post-install configurator the admin runs.
- **Scope:** system-wide — `/usr/bin/mde`, configs to `/etc/skel` (future users)
  + the chosen user's `~/.config`, assets to `/usr/share/mde`.
- **Requirements:** install **everything** — core runtime (sway, foot, swaybg,
  grim, wmenu, NetworkManager, **greetd** + greeter, PipeWire/WirePlumber,
  fonts, xkeyboard-config) **and all 40 system tools**, no prompting.
- **Assets:** **fetched at runtime, never bundled** (locked decision #7). The
  RPM ships code only — the binary plus the asset *installer scripts*
  (`/usr/share/mde/scripts/`). Because the Win2k step deploys per user
  (`~/.local/share`, reading the cached tarball from `~/.config/sway`), the
  system installer deploys the config tree first (step 3) and then triggers the
  per-user fetch via `runuser -u <user> -- mde install --assets`; other users
  fetch on first login. This keeps third-party art (Chicago95 GPL-3, Win2k
  KDE-Store) out of the package — no redistribution. Offline installs can
  pre-seed the tarball cache or point `$MDE_WIN2K_URL` at a local mirror.
- **Login manager:** **greetd**. Register a `MDE-Retro` Wayland session
  (`/usr/share/wayland-sessions/mde-retro.desktop` → `sway`), configure greetd.
- **User/session:** pick an existing user, set MDE-Retro as their default
  session; **no auto-login** (greeter still prompts for password).
- **Finish:** `systemctl set-default graphical.target` + `systemctl enable --now
  greetd` → the console drops straight into the MDE-Retro greeter.

### TUI screens
1. Welcome — "Welcome to Setup… press ENTER to continue, F3 to quit."
2. Summary — what will be installed/configured; ENTER begins.
3. Progress — step list + gauge; runs dnf / deploy / assets / session / finalize.
4. Finish — "MDE-Retro has been installed. The graphical environment will start."

### Install steps (the brain, shared)
1. Collecting information (root check, detect Fedora, network)
2. Installing packages (`dnf install -y` core + 40 tools)
3. Deploying configuration (`/etc/skel`, user home, `/usr/share/mde`)
4. Installing visual assets (per-user fetch: `runuser -u <user> -- mde install
   --assets` → Chicago95 + Win2k into that user's `~/.local/share`)
5. Registering session + greetd (wayland-session, greetd config)
6. Finalizing (`set-default graphical.target`, `enable --now greetd`)

## GUI (in-session path — already built: `installer.rs`)
The Win2000 GUI Setup screen (blue gradient, stage list, progress trough). Same
6-stage brain; runs the unprivileged steps and `pkexec` for dnf.
