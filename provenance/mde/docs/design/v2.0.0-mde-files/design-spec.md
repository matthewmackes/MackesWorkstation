# MDE Files — Rust implementation contract

**Source:** `upstream-bundle/Artifact-Manager.html` (the React prototype).
This document is the Rust translation. When the two disagree, the
prototype wins unless an override note appears below.

---

## 1. Window + chrome

| Property         | Value                                       |
|------------------|---------------------------------------------|
| Window size      | `min(1480px, 94vw) × min(940px, 92vh)`      |
| Titlebar height  | 32 px                                       |
| Sidebar width    | 248 px                                      |
| Window border    | 1 px `rgba(255,255,255,0.08)`               |
| Window shadow    | `0 30px 80px rgba(0,0,0,0.55)` + `0 4px 12px rgba(0,0,0,0.4)` |
| Background       | radial gradient `#2a1408 → #14100a → #0a0806` (warm-dark) |
| Titlebar bg      | `--pf-bg-200` = `#1b1d21`                   |
| Body bg          | `--pf-bg-300` = `#1f1f1f`                   |
| Sidebar bg       | `--window-side` = `#252527`                 |
| Divider color    | `rgba(255,255,255,0.08)`                    |

Titlebar layout: `32 px icon-cell | 1fr title | auto win-controls`.

Title text: `Artifact Manager` (bold `--fg`) + `mesh up · {online}/{total} peers` in mono with a 6 px online dot.

Win-controls: three 46×32 buttons (minimize / maximize / close). Close hover = `#e81123` red. All glyphs from `icons.rs`.

---

## 2. Color tokens

```text
--pf-bg-100   #151515
--pf-bg-200   #1b1d21
--pf-bg-300   #1f1f1f
--pf-bg-400   #292929
--pf-border   #444548
--pf-text-100 #f0f0f0
--pf-text-200 #b8bbbe
--pf-text-300 #8a8d90

--accent      #f0ab00    (amber)
--accent-hi   #ffc107    (amber-hi)
--rust        #e36b3a    (rust accent, used for "self" peer + active sidebar item)

--pf-info     #2b9af3
--pf-success  #3e8635
--pf-danger   #c9190b
```

Status dot color map: `online=#3e8635`, `idle=#f0ab00`, `offline=#444548`, `self=#e36b3a`.

---

## 3. Typography

- Body: **Red Hat Text** 400/500/600/700.
- Headings + window title: **Red Hat Display** 400/500/600/700.
- Mono (peer hosts, sizes, ages, addresses, code identifiers): **Red Hat Mono** 400/500/600.
- Letter-spacing on caps section headers: `0.18em`.

Sidebar section headers are 10 px caps `--fg-faint`. MESH section header is `--accent-hi`.

File rows are 13 px body. File meta (size / age) is 11 px mono `--fg-dim`. Sidebar row text is 13 px body with 12 px mono for the peer host.

---

## 4. Data model

Ported verbatim from the FM_* consts at the top of the prototype.

### Peer

| Field   | Type             | Example                  |
|---------|------------------|--------------------------|
| id      | `&'static str`   | `"pine"`                 |
| host    | `&'static str`   | `"pine.mesh"`            |
| label   | `&'static str`   | `"matthew · workstation"`|
| kind    | `PeerKind`       | `PeerKind::Desktop`      |
| addr    | `&'static str`   | `"10.0.7.14"`            |
| status  | `PeerStatus`     | `PeerStatus::Online`     |
| latency | `Option<u32>`    | `Some(14)` (ms)          |
| files   | `u32`            | `4912`                   |
| shared  | `u32`            | `211`                    |
| last    | `&'static str`   | `"now"` / `"2 h ago"`    |
| derp    | `&'static str`   | `"fra"` / `"ord"`        |

### SelfNode

```rust
SelfNode { id: "yew", host: "yew.mesh", label: "this node", addr: "10.0.7.1", files: 1284, shared: 38 }
```

### FileRow

| Field | Type            |
|-------|-----------------|
| name  | `&'static str`  |
| mime  | `Mime`          |
| size  | `&'static str`  |
| age   | `&'static str`  |
| mesh  | `Option<&str>`  | (peer host that delivered the file, if it came over mesh) |
| from  | `Option<&str>`  | (inbox-style source attribution) |

### Enums

```rust
enum PeerStatus { Online, Idle, Offline, Self_ }
enum PeerKind   { Desktop, Server, Phone, Ci }
enum Mime       { Folder, Doc, Image, Pdf, Archive, Disk }
enum View {
    MeshOverview,        // default landing
    Inbox,
    Peer(PeerId),        // PeerId = &'static str
    Downloads,
    Local,
}
enum Layout { List, Grid }
```

---

## 5. Sidebar (248 px, top-down)

1. **Top toolbar (6 px padding, 28 px button height):**
   - `panelRight` (toggle sidebar)
   - `arrowLeft` (back → resets View to MeshOverview)
   - flex spacer
   - `refresh` (refresh mesh)
2. **MESH section (scrollable, dominates):**
   - Section header `◆ Mesh` (accent-hi caps) with meta `{online}/{total} peers`.
   - "Network overview" item (active when `View == MeshOverview`).
   - Self entry: `<peer-status self>` + `yew.mesh` rust-colored mono + `· you` muted + count.
   - Peer entries: status dot + host + ` · {latency}ms` mono (or omit if offline) + shared count.
   - Inbox item with current inbox count.
   - Outbox item (count = 0 in demo data).
3. **LOCAL section (pinned at bottom via `margin-top: auto` on the foot, see CSS):**
   - Subdued section header `Local` / `this device` (muted, not accent-colored).
   - **Downloads** item — primary class (amber tint, amber left-border). Count = `FM_DOWNLOADS.len()`.
   - "Browse filesystem… /" disclosure (dashed border, 12 px mono). Toggles `local_open`. When open: switches View to `Local`.
   - When `local_open == true`: render 8 dimmed local pins (Home / Documents / Pictures / Music / Videos / Code / Filesystem / Trash). Clicking any of them keeps `View::Local`.
4. **Foot (11 px mono, 1 px top divider):**
   - Left: `tailnet · 10.0.7.0/24`.
   - Right: `[+ Peer]` button (amber-tinted).

### Sidebar row spec

```text
grid-template-columns: 18px 1fr auto;
gap: 10px;
padding: 5px 14px;
font-size: 13px;
hover: bg rgba(255,255,255,0.05) + fg --fg-100;
active: bg rgba(227,107,58,0.16) + fg #fff + border-left 2px var(--rust);
primary (Downloads): bg rgba(240,171,0,0.06) + border-left 2px rgba(240,171,0,0.55);
primary active: bg rgba(240,171,0,0.18) + border-left var(--accent-hi);
dim (filesystem disclosure children): fg --fg-faint, opacity 0.85;
peer.offline: fg --fg-faint + dot opacity 0.6;
```

---

## 6. Main area (right column)

### 6.1 Toolbar (top, 8×16 padding, `--pf-bg-200`)

`crumbs | spacer | search-mini (220 px) | view-toggle (List/Grid) | primary action`.

- **Crumbs** mono 12 px, separator `/`. MESH-flavored crumbs are `--accent-hi`. Trailing chip: `MESH` (amber border) or `LOCAL` (neutral) per current view.
- **Search**: `[icon] [input]`, placeholder `Search mesh…` when in a mesh view, otherwise `Search…`. Focus border `rgba(240,171,0,0.45)`.
- **View toggle**: list / grid, active state amber tint.
- **Primary action**:
  - Mesh views → `[send] Send` (solid amber bg, `#1a1206` text).
  - Downloads → `[upload] Share` (same solid amber).
  - Other → `[folder] New` ghost (transparent + 1 px divider border).

### 6.2 Content area (`fm-content`, 18 × 22 × 28 padding, scrollable)

Five views:

#### 6.2.1 MeshOverview (default landing)

- **Banner** (amber gradient + 3 px amber left border):
  - 40 px icon (meshHub).
  - Title `Mesh is up · {online} of {total} peers reachable`.
  - Subtitle (mono): `tailnet · {self.host} ({self.addr}) · DERP fra · {self.shared} of {self.files} files shared by this node`.
  - Stats column (right): `{online} Online`, `{totalShared} Shared`. Big numbers in `--accent-hi`.
- **Section header**: `Peers · {N}` (caps), right hint `tailnet · sorted by latency`.
- **PeerCard grid** (`auto-fill, minmax(232px, 1fr)`, gap 10 px).
- **Section header**: `Recent mesh transfers` / `last 24 h`.
- **Transfer log**: each row = `[dir-pill] [name] [peer-host] [size · age]`. `dir: in` is info-blue, `dir: out` is amber.

#### 6.2.2 PeerFolder (`View::Peer(id)`)

- Banner: peer kind icon, title `{status-dot} {host} · {label}`, sub `{addr} · {latency} ms via {derp} · {shared} files shared`.
- Stats: `Total files`, `Shared`.
- File list table: `[ico] [name] [origin-pill] [size] [modified]` — every row tagged with the peer host, so every origin pill is the amber mesh pill.

#### 6.2.3 Inbox

- Banner: inbox icon + `Mesh inbox` + sub `files peers sent to {self.host} · auto-routed to ~/mesh/inbox/`.
- Stats: `Items`, `From peers` (unique sender count).
- File list (with `From` pills — same amber pill, source = sender host).

#### 6.2.4 Downloads

- Banner: download icon + `Downloads · ~/Downloads` + sub `local downloads · {mesh-count} items arrived via mesh transfer`.
- Stats: `Items`, `From mesh`.
- File list (mixed pills — items with `mesh: Some(host)` show the amber pill; the rest show `local` neutral pill).

#### 6.2.5 Local (the veil)

**This is deliberately not a folder listing.** It renders as an explainer card:

- Heading: `[hdd] Local filesystem [private to {self.host}]` (chip pill).
- Paragraph (max 64ch): "This is the unsynced filesystem on `{self.host}`. Nothing here is visible to other peers. To share, move a file into `~/mesh` or drag it onto a peer in the sidebar."
- 8-pin grid (`auto-fill, minmax(150px, 1fr)`): each pin = `[icon] {name} [path]`.
- Then a "Recent locally-modified" section header.
- Then 4 sample file rows (all `local` pill).

---

## 7. PeerCard

```text
232 px+ wide, padding 14 14 10
border 1 px --divider; hover border-color rgba(240,171,0,0.4)
self peer: border rgba(227,107,58,0.4); self stripe = rust
online stripe = success-green; idle stripe = amber; offline stripe = pf-border, card opacity 0.55
```

Layout:

- 2 px top stripe (status-colored).
- Header row: 38 px avatar (kind icon) + identity (host mono + label muted).
- Numbers row (mono): `{files} Files`, `{shared} Shared`.
- Meta row (mono 10 px): addr + `{latency} ms · {derp}` (latency colored: <50 green, <150 amber, >=150 default) — OR `last seen {last}` if offline.
- Actions row (1 px divider top): `[Browse →]` primary amber + `[Send file]` ghost + `[•••]` more.

---

## 8. File row

```text
grid-template-columns: 22px 1fr auto 120px 100px;
gap: 12px; padding: 7px 8px; font-size: 13px;
border-bottom: 1px rgba(255,255,255,0.03)
hover: rgba(255,255,255,0.03)
mesh row: bg rgba(240,171,0,0.025), hover rgba(240,171,0,0.06), ico --accent
```

Columns: `[mime icon] [name] [origin pill] [size mono] [age mono]`. Heading row uses caps 10 px mono.

Origin pill:
- **mesh-pill** (`↘ peer.mesh`): `bg rgba(240,171,0,0.10)`, color `--accent-hi`, border `rgba(240,171,0,0.25)`, arrow color `--rust`.
- **local-pill** (`local`): `bg rgba(255,255,255,0.03)`, color `--fg-faint`, border `rgba(255,255,255,0.06)`.

---

## 9. Icons

Every icon is **Lucide-style** — 24-grid, fill=none, stroke=currentColor, strokeWidth 1.6, strokeLinecap=square, strokeLinejoin=miter. Captured verbatim from the prototype in `crates/mde-files/src/icons.rs`. Names:

`panelRight, arrowLeft, refresh, search, plus, more, minus, maximize, close, chevronRight, chevronDown, folder, doc2, imageFile, pdf, archive, diskImg, meshHub, monitor, server, phone, cpu, inbox, send, download, upload, listView, gridView, home, hdd, trash2, player, doc, rust`.

Rendered with `iced::widget::svg::Svg::new(svg::Handle::from_memory(SVG_BYTES))`.

---

## 10. Demo data

Embedded as `const` arrays in `crates/mde-files/src/demo_data.rs`. The
exact values from the prototype (FM_PEERS / FM_SELF / FM_RECENT /
FM_INBOX / FM_DOWNLOADS / FM_PINE_FILES / FM_BIRCH_FILES /
FM_OAK_FILES / FM_LOCAL_PINS) are reproduced. Demo data is what the
app shows until `mded` (Phase A of v2.0.0) is wired in; at that point
the const arrays get swapped for a `Backend` trait implementation
backed by zbus calls.

---

## 11. View state machine

```rust
struct State {
    view: View,           // default View::MeshOverview
    local_open: bool,     // default false
    layout: Layout,       // default Layout::List
    search: String,       // default ""
}

enum Message {
    SelectView(View),
    ToggleLocal,
    SetLayout(Layout),
    SearchChanged(String),
    Refresh,
    TitlebarMinimize,
    TitlebarMaximize,
    TitlebarClose,
    PeerCardBrowse(PeerId),
    PeerCardSend(PeerId),
}
```

Disclosure interaction (the trickiest bit, ported from the prototype):

```text
on disclosure click:
  local_open = !local_open
  if local_open AND view != Local: view = Local
  if !local_open AND view == Local: view = MeshOverview
```

When `local_open == true` AND `view == Local`, clicking any of the 8 local pins keeps `view = Local` (the pin click is essentially a no-op in this prototype; later it'll open a flat folder listing under the chosen pin).

---

## 12. Backend trait (forward-looking)

The Rust crate **starts** with `Backend::Demo` that serves the const
arrays. Once `mded` lands, a `Backend::DBus` implementation will:

- Subscribe to `dev.mackes.MDE.Fleet.Peers` for peer state.
- Subscribe to `dev.mackes.MDE.Shell.FileOperations` for transfer events.
- Call `dev.mackes.MDE.Shell.Inbox`/`Outbox`/`Downloads` for the file lists.
- Call `dev.mackes.MDE.Fleet.SendFile(peer, path, mode, conflict_policy)` for the primary action.

This is captured in the worklist as the cosmic-files-fork-merge phase.

## 13. Out-of-scope (for the prototype, future worklist items)

The prototype intentionally **does not** show:
- File previews (still future).
- Multi-select + bulk actions UI (data model supports it; UI deferred).
- Drag-and-drop visual states.
- Per-row context menu (Send To submenu).
- Operation drawer (progress + cancel + retry + rollback UI).
- Search results.
- Audit-trail / operation-history panel.

These all live as `[ ] Open` items under the MDE Files worklist
section. Phase 0 of MDE Files is "ship the prototype's surface" —
later phases extend it with the cross-platform Send-To matrix from the
prior MAP2 plan (toolbar/menu/palette/DnD/details/bulk × destinations ×
modes × conflict-policies).
