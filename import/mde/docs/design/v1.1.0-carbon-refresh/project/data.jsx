/* Realistic mock state for the Mackes Shell prototype */

const PRESETS = [
  {
    name: "hash", display: "#!", subtitle: "CrunchBang reincarnation",
    voice: "Black, monospace, sparse. Modern stack: alacritty / neovim / firefox / mpv.",
    accent: "#f4f4f4",
    swatches: ["#0a0a0a", "#161616", "#f4f4f4"],
    bgGradient: "linear-gradient(135deg, #0a0a0a 0%, #161616 100%)",
  },
  {
    name: "mackes", display: "Mackes", subtitle: "Warm-dark house style",
    voice: "Curated dev toolset: VS Code, Cursor, Claude Code CLI, Terminator, FileZilla, Remmina.",
    accent: "#f1853d",
    swatches: ["#1a1410", "#2a1d12", "#f1853d"],
    bgGradient: "linear-gradient(135deg, #1a1410 0%, #2a1d12 60%, #3a2418 100%)",
  },
  {
    name: "daylight", display: "Daylight", subtitle: "Cool yellow accent",
    voice: "Productivity stack: LibreOffice, Thunderbird, GIMP, Inkscape, Evince.",
    accent: "#fddc69",
    swatches: ["#161616", "#262626", "#fddc69"],
    bgGradient: "linear-gradient(135deg, #161616 0%, #2a2920 60%, #3a3826 100%)",
  },
  {
    name: "vanilla", display: "Vanilla", subtitle: "Fedora XFCE defaults",
    voice: "Mackes manages snapshots + repair only — never touches your theme, panel, or app set.",
    accent: "#4589ff",
    swatches: ["#2e3436", "#3465a4", "#eeeeec"],
    bgGradient: "linear-gradient(135deg, #2e3436 0%, #2e4878 60%, #3465a4 100%)",
  },
];

const PEERS = [
  { id: "kiln",     name: "kiln.mesh",     ip: "100.64.0.1",  route: "DERP/direct", rtt: 0,  lastSeen: "now",        online: true, control: true,  os: "Fedora 41 Server", role: "control · fileserver", x: 0.5, y: 0.5 },
  { id: "anvil",    name: "anvil.mesh",    ip: "100.64.0.2",  route: "direct",      rtt: 2,  lastSeen: "now",        online: true, control: false, os: "Fedora 41 Workstation", role: "dev workstation", x: 0.18, y: 0.22 },
  { id: "forge",    name: "forge.mesh",    ip: "100.64.0.3",  route: "direct",      rtt: 4,  lastSeen: "now",        online: true, control: false, os: "Fedora 41 Workstation", role: "secondary dev", x: 0.82, y: 0.22 },
  { id: "vault",    name: "vault.mesh",    ip: "100.64.0.4",  route: "direct",      rtt: 1,  lastSeen: "now",        online: true, control: false, os: "Fedora 41 (NAS)", role: "media NAS · 24 TB", x: 0.18, y: 0.78 },
  { id: "lantern",  name: "lantern.mesh",  ip: "100.64.0.5",  route: "DERP",        rtt: 34, lastSeen: "now",        online: true, control: false, os: "Fedora 41 (laptop)", role: "travel laptop", x: 0.82, y: 0.78 },
  { id: "ember",    name: "ember.mesh",    ip: "100.64.0.6",  route: "DERP",        rtt: 87, lastSeen: "12m ago",    online: false, control: false, os: "Fedora 41 (VPS)", role: "Hetzner FSN1 VPS · public", x: 0.5, y: 0.92 },
];

const SERVICES = [
  { id: "jellyfin",  name: "Jellyfin",          peer: "vault",   url: "http://vault.mesh:8096",   kind: "Media",        status: "ok" },
  { id: "airsonic",  name: "Airsonic Advanced", peer: "vault",   url: "http://vault.mesh:4040",   kind: "Music",        status: "ok" },
  { id: "sonarr",    name: "Sonarr",            peer: "vault",   url: "http://vault.mesh:8989",   kind: "Automation",   status: "ok" },
  { id: "radarr",    name: "Radarr",            peer: "vault",   url: "http://vault.mesh:7878",   kind: "Automation",   status: "ok" },
  { id: "grafana",   name: "Grafana",           peer: "kiln",    url: "http://kiln.mesh:3000",    kind: "Observability", status: "ok" },
  { id: "homeasst",  name: "Home Assistant",    peer: "kiln",    url: "http://kiln.mesh:8123",    kind: "Smart Home",   status: "ok" },
  { id: "syncthing", name: "Syncthing",         peer: "anvil",   url: "http://anvil.mesh:8384",   kind: "Sync",         status: "ok" },
  { id: "code-server", name: "code-server",     peer: "forge",   url: "https://forge.mesh:8443",  kind: "Dev",          status: "ok" },
  { id: "gitea",     name: "Gitea",             peer: "kiln",    url: "http://kiln.mesh:3030",    kind: "Dev",          status: "ok" },
  { id: "minio",     name: "MinIO",             peer: "kiln",    url: "https://kiln.mesh:9001",   kind: "Storage",      status: "warn" },
  { id: "prometheus",name: "Prometheus",        peer: "kiln",    url: "http://kiln.mesh:9090",    kind: "Observability", status: "ok" },
  { id: "nextcloud", name: "Nextcloud",         peer: "vault",   url: "https://vault.mesh:443",   kind: "Files",        status: "ok" },
];

const APPS_CATALOG = [
  { id: "vscode", name: "Visual Studio Code", icon: "VS", category: "Development", desc: "Microsoft's polyglot editor.", size: "84 MB", installed: true, preset: ["mackes"] },
  { id: "cursor", name: "Cursor", icon: "CR", category: "Development", desc: "AI-pair editor, VS Code fork.", size: "112 MB", installed: true, preset: ["mackes"] },
  { id: "claude-code", name: "Claude Code CLI", icon: "CC", category: "Development", desc: "Anthropic's terminal-native coding agent.", size: "26 MB", installed: true, preset: ["mackes"] },
  { id: "terminator", name: "Terminator", icon: "TT", category: "Terminal", desc: "Tiled terminal emulator with profiles.", size: "4 MB", installed: true, preset: ["mackes"] },
  { id: "alacritty", name: "Alacritty", icon: "AL", category: "Terminal", desc: "GPU-accelerated terminal.", size: "3 MB", installed: true, preset: ["hash"] },
  { id: "neovim", name: "Neovim", icon: "NV", category: "Editor", desc: "Hyperextensible Vim-based editor.", size: "12 MB", installed: true, preset: ["hash"] },
  { id: "firefox", name: "Firefox", icon: "FX", category: "Browser", desc: "Mozilla's standards-first browser.", size: "240 MB", installed: true, preset: ["hash","daylight"] },
  { id: "edge", name: "Microsoft Edge", icon: "ME", category: "Browser", desc: "Chromium-based.", size: "320 MB", installed: false, preset: ["mackes"] },
  { id: "mpv", name: "mpv", icon: "MV", category: "Media", desc: "Minimal media player.", size: "6 MB", installed: true, preset: ["hash"] },
  { id: "libreoffice", name: "LibreOffice", icon: "LO", category: "Office", desc: "Free office suite.", size: "412 MB", installed: false, preset: ["daylight"] },
  { id: "thunderbird", name: "Thunderbird", icon: "TB", category: "Mail", desc: "Mozilla mail and calendar.", size: "98 MB", installed: false, preset: ["daylight"] },
  { id: "gimp", name: "GIMP", icon: "GP", category: "Graphics", desc: "Raster image editor.", size: "210 MB", installed: false, preset: ["daylight"] },
  { id: "inkscape", name: "Inkscape", icon: "IK", category: "Graphics", desc: "Vector graphics editor.", size: "180 MB", installed: false, preset: ["daylight"] },
  { id: "filezilla", name: "FileZilla", icon: "FZ", category: "Network", desc: "FTP/SFTP client.", size: "14 MB", installed: true, preset: ["mackes"] },
  { id: "remmina", name: "Remmina", icon: "RM", category: "Network", desc: "Remote desktop client.", size: "18 MB", installed: true, preset: ["mackes"] },
  { id: "conky", name: "Conky", icon: "CK", category: "System", desc: "Lightweight system monitor.", size: "2 MB", installed: true, preset: ["hash"] },
];

const SNAPSHOTS = [
  { id: "s1", name: "before-theme-swap", created: "2026-05-17 09:14", preset: "Mackes", size: "184 KB" },
  { id: "s2", name: "pre-drift-review", created: "2026-05-16 22:01", preset: "Mackes", size: "182 KB" },
  { id: "s3", name: "post-1.0.0-upgrade", created: "2026-05-14 11:30", preset: "Mackes", size: "176 KB" },
  { id: "s4", name: "fresh-install", created: "2026-05-12 18:42", preset: "Mackes", size: "158 KB" },
];

const RECENT_ACTIVITY = [
  { t: "09:14", what: "Theme set to PadOS-Dark", who: "Appearance" },
  { t: "09:14", what: "Created snapshot before-theme-swap", who: "Snapshots" },
  { t: "08:52", what: "Added peer lantern.mesh (100.64.0.5)", who: "Mesh VPN" },
  { t: "08:21", what: "Installed Cursor 0.49.6", who: "Apps" },
  { t: "08:12", what: "Mounted vault.mesh:/media at ~/QNM-Mesh/vault", who: "Mesh FS" },
  { t: "yesterday 22:01", what: "Snapshot pre-drift-review created", who: "Snapshots" },
];

const HARDWARE = {
  hostname: "anvil",
  os: "Fedora Linux 41 (Workstation Edition) · XFCE 4.20",
  cpu: "AMD Ryzen 9 7950X · 16C/32T · 4.5 GHz",
  ram: "64 GiB DDR5 · 38% used",
  gpu: "NVIDIA RTX 4070 · driver 550.78",
  disk: "Samsung 990 Pro 2 TB · 612 GiB free",
  uptime: "3d 14h",
};

const DRIFT = [
  { section: "xsettings", field: "/Net/ThemeName", expected: "'PadOS-Dark'", actual: "'Adwaita-dark'" },
  { section: "xfce4-panel", field: "/panels/panel-1/size", expected: "28", actual: "32" },
  { section: "xfwm4", field: "/general/theme", expected: "'PadOS'", actual: "'Default'" },
];

const HELP_TOPICS = [
  { id: "getting-started", title: "Getting started", section: "Basics" },
  { id: "dashboard", title: "Dashboard", section: "Basics" },
  { id: "presets", title: "Presets", section: "Basics" },
  { id: "look-and-feel", title: "Look & Feel", section: "Configuration" },
  { id: "devices", title: "Devices", section: "Configuration" },
  { id: "network", title: "Network", section: "Configuration" },
  { id: "system", title: "System", section: "Configuration" },
  { id: "apps", title: "Apps", section: "Configuration" },
  { id: "mesh", title: "Mesh overview", section: "Mesh" },
  { id: "mesh-vpn", title: "Mesh VPN", section: "Mesh" },
  { id: "mesh-ssh", title: "Mesh SSH", section: "Mesh" },
  { id: "mesh-services", title: "Mesh services", section: "Mesh" },
  { id: "mesh-thunar", title: "Mesh in Thunar", section: "Mesh" },
  { id: "maintain", title: "Maintain", section: "Operations" },
  { id: "headless", title: "Headless mode", section: "Operations" },
  { id: "keybindings", title: "Keybindings", section: "Reference" },
  { id: "cli-reference", title: "CLI reference", section: "Reference" },
  { id: "troubleshooting", title: "Troubleshooting", section: "Reference" },
  { id: "index", title: "All topics", section: "Reference" },
];

Object.assign(window, { PRESETS, PEERS, SERVICES, APPS_CATALOG, SNAPSHOTS, RECENT_ACTIVITY, HARDWARE, DRIFT, HELP_TOPICS });
