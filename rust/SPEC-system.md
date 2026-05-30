# System Properties + Device Manager spec

Replicates the Windows XP **System Properties** dialog (ref:
sfu.ca/.../rdc_fig1.jpg), rendered in the MDE-Retro Win2000 classic theme. All
7 tabs, each wired to its Fedora equivalent. Opened from the Control Panel
"System" item (and Win+Pause).

## Tabs (XP layout, two rows)
General · Computer Name · Hardware · Advanced · System Restore · Automatic
Updates · Remote

| Tab | Wiring (decided) |
|-----|------------------|
| **General** | Live system info: `/etc/os-release`, kernel, `lscpu` CPU, `/proc/meminfo` RAM, "Registered to" = user. |
| **Computer Name** | Hostname + description; **Change…** sets hostname via `hostnamectl` (pkexec); a Samba **Workgroup** field. |
| **Hardware** | **Device Manager = vendored HardInfo2** (`vendor/hardinfo2`), themed Win2000 (Chicago95 GTK) and launched as "Device Manager". A **[Device Manager]** button opens it. |
| **Advanced** | Full set: **Environment Variables** (user + system), **Performance** (zram/swappiness), **Startup & Recovery** (default boot entry + GRUB timeout). |
| **System Restore** | → **Timeshift** (enable/create snapshots).  *[confirm in Q5]* |
| **Automatic Updates** | → **dnf-automatic** (enable timer, configure).  *[confirm in Q6]* |
| **Remote** | **Remote Desktop** = wayvnc (allow users to connect); **Remote Assistance** = one-time invite.  *[details in Q7–10]* |

## Device Manager (= HardInfo2, vendored)
- **Source imported** as submodule `vendor/hardinfo2` (GPL-2.0+, GTK3/cmake).
- Becomes the platform Device Manager: build it, **theme it Win2000** (Chicago95
  GTK theme + CSS), launch as "Device Manager" from the Hardware tab and the
  System Tools menu.
- Earlier "build our own MS-style tree" answers (data source = lshw, MS naming,
  4 view modes, status icons) are **superseded** — HardInfo provides the engine
  and UI. Remaining effort = theming + relabeling toward MS naming where
  practical (refinement, not a from-scratch build).
- Decided before the pivot (kept for theming reference): MS-style category names,
  all four view modes, status icons (no-driver = ⚠, disabled = ↓), Show hidden.

## Remote Desktop (Wayland)
- Backend: **wayvnc** (wlroots/sway VNC). Install via pkexec dnf if missing.
- "Allow users to connect remotely" toggles the wayvnc service; show the
  connection address (host:5900). *[security/users details pending Q7–10]*

## Still to confirm (System Properties Q5–10)
System Restore backend, Automatic Updates backend, Remote Desktop vs RDP,
Select Remote Users semantics, Remote Assistance, and security (password/TLS,
localhost+SSH vs LAN, port). Answer while building those tabs.
