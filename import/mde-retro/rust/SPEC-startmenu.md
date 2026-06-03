# Start menu + shell interaction spec (from 39 design Q&A, 2026-05-30)

Authoritative behavior for `mde menu`, the taskbar context actions, and the
launcher right-click menu. Build to this.

## Left-click Start
- **Toggle** open/close. Also opens via the **Super (Win) key** (Ctrl+Esc free).
- Opens the **authentic classic column** (not the old flat list).
- Submenus **open on click** (not hover).
- **Personalized menus OFF** — always show all items.
- **Close** on: click-outside, Esc, or launching an item (launch always closes).
- **Full keyboard nav**: Up/Down move, Right/Enter open submenu/activate,
  Left/Esc go back a level, underlined **accelerator letters** jump.
- Side **banner**: "MDE-Retro" rotated white bold on a blue gradient.

### Main column (top → bottom)
```
[ Pinned items ]            (user-pinned launchers, persisted)
─────
Windows Update
─────
Programs        ▶          installed .desktop apps, grouped by category
Settings        ▶          Control Panel · Network and Dial-up Connections ·
                            Printers · Taskbar & Start Menu
Search          ▶          For Files or Folders... · On the Internet...
System Tools    ▶          the 40 Fedora tools by category (fedora.rs)
Help                       opens docs/man
Run...                     reuse wofi run
─────
Log Off...                 confirm dialog (Yes/No) → swaymsg exit
Shut Down...               dialog w/ dropdown: Log off / Shut down / Restart /
                            Stand by  + OK/Cancel
```
- Documents: **omitted**.

## Right-click Start button
- Opens the **System Tools menu** (the Fedora-tools menu — current behavior).

## Right-click a launcher (menu item)
Launch (left-click or Open) = **run + close menu**.

**Installed tool:**
```
Open
Run as administrator        (pkexec — on EVERY launcher)
─────
Pin                         pin to the PANEL (Quick Launch), persisted
Pin to Start menu           add to the pinned area at top of Start
Send To            ▶        Desktop (create shortcut) · Documents
─────
Open file location          opens the .desktop's folder in mde files
Copy command                exec line → clipboard
─────
Rename                      custom display name (persisted override)
Uninstall                   pkexec dnf remove (confirm dialog first)
─────
Properties                  full Win2000-style dialog (see below)
```
**Missing tool:**
```
Install
Install and Run
─────
Properties
```

### Properties dialog (full Win2000 style)
Fields: Name, Target command, Package, Category, Install status, Icon +
**Change Icon...** (pick a freedesktop icon, persisted override). OK/Cancel.

## Taskbar
- Add a **Quick Launch toolbar** between Start and the window buttons:
  Show Desktop + pinned apps.
- **Right-click empty taskbar**: Toolbars ▶, ──, Cascade Windows, Tile Windows,
  Minimize All Windows, ──, Task Manager, ──, Properties.
  - Cascade / Tile / Minimize-All are **wired to sway** (swaymsg).
  - **Task Manager** = launch **btop** in foot at 150%.
- **Right-click a window button**: Restore, Move, Size, Minimize, Maximize, ──,
  Close (the Win2000 system menu), via sway IPC.
- **Right-click the tray clock**: Adjust Date/Time.

## System Properties (the Control Panel "System" item)
Tabbed dialog (Win2000/XP style). Tabs: General, Computer Name, Remote (+ more
as built). **Remote tab** has **"Enable Remote Desktop (Wayland)"** — toggles
**wayvnc** (wlroots/sway VNC server): install via pkexec dnf if missing,
start/stop the service, show the connection address (host:5900).

## Persistence
User overrides — pins (panel + Start), renames, custom icons, hidden items —
persist in `~/.config/mde/menu.json`.

## Build order
1. Font wiring (Droid Sans as Tahoma) — in progress.
2. Authentic Start menu column + click submenus + Programs(.desktop scan) +
   System Tools + Settings/Search flyouts.
3. Dialogs: Log Off, Shut Down, Properties, System Properties (Remote/wayvnc).
4. Launcher right-click context menu + persistence (~/.config/mde/menu.json).
5. Taskbar: Quick Launch, right-click menus, window-button system menu, clock.
6. Keyboard nav + accelerators; Win-key + Start toggle.
7. (Then, only when complete) accuracy harness → RPM.
