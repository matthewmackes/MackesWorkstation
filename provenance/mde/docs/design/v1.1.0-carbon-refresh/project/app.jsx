/* The Carbon UI Shell + XFCE window chrome + main app state */

const NAV = [
  { group: "Workbench", items: [
    { key: "dashboard", label: "Dashboard", icon: "dashboard" },
  ]},
  { group: "Configuration", items: [
    { key: "appearance", label: "Look & Feel", icon: "paint" },
    { key: "devices", label: "Devices", icon: "devices" },
    { key: "system", label: "System", icon: "system" },
  ]},
  { group: "Network", items: [
    { key: "wifi", label: "Wi-Fi & Ethernet", icon: "wifi" },
    { key: "mesh-vpn", label: "Mesh VPN", icon: "mesh", badge: "5" },
    { key: "mesh-ssh", label: "Mesh SSH", icon: "lock" },
    { key: "mesh-services", label: "Mesh Services", icon: "cloud", badge: "12" },
    { key: "firewall", label: "Firewall", icon: "flame" },
  ]},
  { group: "Apps & Maintenance", items: [
    { key: "apps", label: "Apps", icon: "apps" },
    { key: "maintain", label: "Maintain", icon: "wrench" },
    { key: "snapshots", label: "Snapshots", icon: "snapshot" },
  ]},
  { group: "Reference", items: [
    { key: "help", label: "Help", icon: "help" },
  ]},
];

const TWEAK_DEFAULTS = /*EDITMODE-BEGIN*/{
  "preset": "mackes",
  "density": "cozy",
  "showStatusBar": true,
  "showWizard": false,
  "topologyMode": "topology",
  "showXfceFrame": true
}/*EDITMODE-END*/;

const PRESET_LABELS = { hash: "#!", mackes: "Mackes", daylight: "Daylight", vanilla: "Vanilla", node: "Node" };

const App = () => {
  const [t, setTweak] = useTweaks(TWEAK_DEFAULTS);
  const [active, setActive] = useState("dashboard");
  const [toasts, setToasts] = useState([]);
  const [wizardOpen, setWizardOpen] = useState(t.showWizard);
  const toastIdRef = useRef(0);

  useEffect(() => { setWizardOpen(t.showWizard); }, [t.showWizard]);

  const state = {
    preset: t.preset,
    presetLabel: PRESET_LABELS[t.preset] || "Mackes",
  };
  const setState = (s) => {
    if (s.preset && s.preset !== t.preset) setTweak("preset", s.preset);
  };

  const toast = useCallback((message, kind) => {
    const id = ++toastIdRef.current;
    setToasts(ts => [...ts, { id, message, kind }]);
    setTimeout(() => setToasts(ts => ts.filter(x => x.id !== id)), 3200);
  }, []);

  const navigate = useCallback((key) => {
    // map sub-routes back to their parent panel for prototype
    const parentMap = { reset: "maintain", logs: "maintain", repair: "maintain", health: "maintain", drift: "maintain", fonts: "maintain", power: "maintain", resources: "maintain", "system-update": "maintain", dependencies: "maintain", uninstall: "maintain", display: "devices", network: "wifi", keyboard: "devices", mouse: "devices", sound: "devices" };
    setActive(parentMap[key] || key);
  }, []);

  const panel = (() => {
    switch (active) {
      case "dashboard": return <DashboardPanel state={state} navigate={navigate} toast={toast} />;
      case "appearance": return <AppearancePanel state={state} setState={setState} toast={toast} />;
      case "devices": return <DevicesPanel toast={toast} />;
      case "system": return <SystemPanel toast={toast} />;
      case "wifi": return <NetworkPanel toast={toast} />;
      case "mesh-vpn": return <MeshVpnPanel state={state} toast={toast} />;
      case "mesh-ssh": return <MeshSshPanel toast={toast} />;
      case "mesh-services": return <MeshServicesPanel toast={toast} />;
      case "firewall": return <FirewallPanel toast={toast} />;
      case "apps": return <AppsPanel state={state} toast={toast} />;
      case "maintain": return <MaintainPanel navigate={k => setActive(k === "snapshots" ? "snapshots" : "maintain")} />;
      case "snapshots": return <SnapshotsPanel toast={toast} />;
      case "help": return <HelpPanel />;
      default: return <DashboardPanel state={state} navigate={navigate} toast={toast} />;
    }
  })();

  const xfceFrame = t.showXfceFrame !== false;

  return (
    <div className="viewport">
      {xfceFrame ? (
        <div className="xfce-frame">
          <XfceTitlebar />
          <Shell active={active} setActive={setActive} state={state} panel={panel} t={t} />
        </div>
      ) : (
        <div style={{ width: "100%", height: "100%" }}>
          <Shell active={active} setActive={setActive} state={state} panel={panel} t={t} />
        </div>
      )}
      <ToastHost toasts={toasts} />
      {wizardOpen && <Wizard onClose={() => { setWizardOpen(false); setTweak("showWizard", false); }} state={state} setState={setState} toast={toast} />}
      <TweaksWrap t={t} setTweak={setTweak} />
    </div>
  );
};

const XfceTitlebar = () => (
  <div className="xfce-titlebar">
    <div className="row" style={{ gap: 6 }}>
      <div className="xfce-icon" />
      <span>Mackes Shell — Workbench</span>
    </div>
    <div className="xfce-title"></div>
    <div className="xfce-btns">
      <div className="xfce-btn min" />
      <div className="xfce-btn max" />
      <div className="xfce-btn close" />
    </div>
  </div>
);

const Shell = ({ active, setActive, state, panel, t }) => (
  <div className="app-shell" data-preset={t.preset} data-density={t.density}>
    <header className="shell-header">
      <div className="shell-header-brand">
        <div className="brand-logo" />
        <div className="brand-text">Mackes<span className="brand-light">Shell</span></div>
      </div>
      <nav style={{ display: "flex", height: "100%", marginLeft: 16 }}>
        <button className="header-action" style={{ borderLeft: 0 }}>Workbench</button>
        <button className="header-action">Recovery</button>
        <button className="header-action">CLI</button>
      </nav>
      <div className="shell-header-actions">
        <button className="header-action">
          <Icon name="bell" />
          <span>3</span>
        </button>
        <button className="header-action">
          <Icon name="search" />
        </button>
        <button className="header-action">
          <Tag kind="accent">{state.presetLabel}</Tag>
        </button>
        <button className="header-action">
          <Icon name="user" />
          <span>matt@anvil</span>
        </button>
      </div>
    </header>

    <div className="shell-body">
      <aside className="side-nav">
        {NAV.map(group => (
          <div key={group.group} className="side-nav-group">
            <div className="side-nav-group-title">{group.group}</div>
            {group.items.map(item => (
              <button key={item.key} className="side-nav-item" data-active={active === item.key} onClick={() => setActive(item.key)}>
                <span className="sn-icon"><Icon name={item.icon} /></span>
                <span className="sn-label">{item.label}</span>
                {item.badge && <span className="sn-badge">{item.badge}</span>}
              </button>
            ))}
          </div>
        ))}
      </aside>

      <main className="content">
        {panel}
      </main>
    </div>

    {t.showStatusBar && (
      <footer className="shell-status">
        <span className="sb-item"><span className="dot ok" /> mesh: 5/16</span>
        <span className="sb-item"><span className="dot ok" /> services: 12</span>
        <span className="sb-item"><span className="dot ok" /> sshd</span>
        <span className="sb-item"><span className="dot warn" /> drift: 3</span>
        <span className="sb-item right">v1.0.0 · build 2026.05.17</span>
        <span className="sb-item">cpu 12% · ram 38%</span>
        <span className="sb-item">anvil.mesh / 100.64.0.2</span>
      </footer>
    )}
  </div>
);

// ============================================================
// Lightweight stub panels for sections not built out
// ============================================================
const DevicesPanel = ({ toast }) => {
  const subs = [
    { id: "display", title: "Display", icon: "monitor", desc: "Resolution, scaling, multi-monitor.", meta: "3840×2160 @ 60 Hz" },
    { id: "keyboard", title: "Keyboard", icon: "keyboard", desc: "Layout, repeat rate, shortcuts.", meta: "US · 30 ms" },
    { id: "mouse", title: "Mouse & Touchpad", icon: "mouse", desc: "Speed, acceleration, tap-to-click.", meta: "1.4x speed" },
    { id: "sound", title: "Sound", icon: "sound", desc: "Output device, input device, levels.", meta: "Realtek ALC1220" },
    { id: "power", title: "Power", icon: "power", desc: "Suspend, lid actions, brightness.", meta: "Never suspend" },
  ];
  return (
    <div className="content-inner">
      <div className="breadcrumbs"><span>Mackes Shell</span><span className="sep">/</span><span>Devices</span></div>
      <h1 className="page-title">Devices</h1>
      <p className="page-subtitle">All hardware-facing settings, backed by xfconf channels and live xrandr/PipeWire/UPower.</p>
      <div className="grid-3">
        {subs.map(s => (
          <Tile key={s.id} clickable onClick={() => toast(`Opened ${s.title}`)}>
            <Icon name={s.icon} size={20} color="var(--accent)" />
            <div style={{ font: "var(--type-heading-02)" }}>{s.title}</div>
            <div className="muted">{s.desc}</div>
            <div className="mono" style={{ color: "var(--text-helper)", marginTop: 4 }}>{s.meta}</div>
          </Tile>
        ))}
      </div>
    </div>
  );
};

const SystemPanel = ({ toast }) => {
  const subs = [
    "Window Manager", "Workspaces", "Session & Startup", "Notifications", "Default Apps", "Removable Media", "Date & Time",
  ];
  return (
    <div className="content-inner">
      <div className="breadcrumbs"><span>Mackes Shell</span><span className="sep">/</span><span>System</span></div>
      <h1 className="page-title">System</h1>
      <p className="page-subtitle">XFCE session, window manager, notifications, defaults.</p>
      <div className="grid-3">
        {subs.map(s => (
          <Tile key={s} clickable onClick={() => toast(s)}>
            <div style={{ font: "var(--type-heading-02)" }}>{s}</div>
            <div className="muted">xfconf-backed; immediate apply.</div>
          </Tile>
        ))}
      </div>
    </div>
  );
};

const NetworkPanel = ({ toast }) => (
  <div className="content-inner">
    <div className="breadcrumbs"><span>Mackes Shell</span><span className="sep">/</span><span>Network</span><span className="sep">/</span><span>Wi-Fi & Ethernet</span></div>
    <h1 className="page-title">Wi-Fi & Ethernet</h1>
    <p className="page-subtitle">Carrier-grade NetworkManager surfaced through Carbon. Static and DHCP, VLAN, captive-portal handoff.</p>
    <SectionH title="Active connections" />
    <DataTable
      columns={[
        { key: "name", title: "Connection", render: r => <span><Dot status={r.status} /> &nbsp;{r.name}</span> },
        { key: "type", title: "Type" },
        { key: "speed", title: "Speed", render: r => <span className="dt-mono">{r.speed}</span> },
        { key: "addr", title: "Address", render: r => <span className="dt-mono">{r.addr}</span> },
        { key: "since", title: "Up since", render: r => <span className="dt-mono">{r.since}</span> },
      ]}
      rows={[
        { id: "eth0", name: "enp7s0 (Ethernet)", type: "wired · 1G", speed: "1.0 Gbps", addr: "192.168.1.42/24", since: "3d 14h", status: "ok" },
        { id: "wlan0", name: "wlp4s0 (Wi-Fi)", type: "802.11ax", speed: "—", addr: "—", since: "—", status: "muted" },
      ]}
    />
  </div>
);

const MeshSshPanel = ({ toast }) => (
  <div className="content-inner">
    <div className="breadcrumbs"><span>Mackes Shell</span><span className="sep">/</span><span>Network</span><span className="sep">/</span><span>Mesh SSH</span></div>
    <h1 className="page-title">Mesh SSH</h1>
    <p className="page-subtitle">Identity-based zero-config SSH between every mesh peer. Ed25519 keys auto-distributed via Headscale Tailscale-SSH.</p>

    <Notif kind="success" title="Tailscale-SSH active on 5 peers" icon="✓">
      ACLs live in <span className="mono">/etc/headscale/acls.hujson</span> · last sync 4s ago.
    </Notif>

    <SectionH title="Peers reachable via SSH" />
    <DataTable
      searchable
      columns={[
        { key: "name", title: "Peer", render: r => <span><Dot status={r.online ? "ok" : "fail"} /> &nbsp;{r.name}</span> },
        { key: "fingerprint", title: "Host key fingerprint", render: r => <span className="dt-mono">SHA256:{r.id.slice(0,4)}...{r.id.slice(-4)}rRkVQ8AzPLm</span> },
        { key: "users", title: "Allowed users", render: () => <Tag>matt</Tag> },
        { key: "cmd", title: "", render: r => <Btn kind="ghost" size="sm" icon="server" onClick={() => toast(`ssh ${r.name}`)}>Open</Btn> }
      ]}
      rows={window.PEERS}
      rowKey="id"
    />

    <SectionH title="Access control" />
    <Tile>
      <div className="code">{`# acls.hujson — identity-based ACLs
{
  "groups": { "group:trusted": ["matt@home"] },
  "acls": [
    { "action": "accept",
      "src":    ["group:trusted"],
      "dst":    ["*:22"] }
  ],
  "ssh": [
    { "action": "check",
      "src":    ["group:trusted"],
      "dst":    ["*"],
      "users":  ["matt"] }
  ]
}`}</div>
    </Tile>
  </div>
);

const FirewallPanel = ({ toast }) => (
  <div className="content-inner">
    <div className="breadcrumbs"><span>Mackes Shell</span><span className="sep">/</span><span>Network</span><span className="sep">/</span><span>Firewall</span></div>
    <h1 className="page-title">Firewall</h1>
    <p className="page-subtitle">firewalld zones, services, ports. Mesh traffic permitted on the <span className="mono">mesh</span> zone.</p>

    <SectionH title="Zones" />
    <div className="grid-3">
      {[
        { name: "public", iface: "—", active: false, services: ["dhcpv6-client", "ssh"] },
        { name: "home", iface: "enp7s0", active: true, services: ["dhcpv6-client", "ssh", "samba-client", "mdns"] },
        { name: "mesh", iface: "tailscale0", active: true, services: ["ssh", "all-mesh-services"], accent: true },
      ].map(z => (
        <Tile key={z.name} style={z.accent ? { borderTop: "2px solid var(--accent)" } : {}}>
          <div className="row between">
            <div style={{ font: "var(--type-heading-02)" }}>{z.name}</div>
            {z.active ? <Tag kind="success">active</Tag> : <Tag>inactive</Tag>}
          </div>
          <div className="muted mono">{z.iface}</div>
          <div className="row" style={{ flexWrap: "wrap", marginTop: 8 }}>
            {z.services.map(s => <Tag key={s}>{s}</Tag>)}
          </div>
        </Tile>
      ))}
    </div>
  </div>
);

// ============================================================
// Tweaks (host protocol; uses tweaks-panel.jsx primitives)
// ============================================================
const TweaksWrap = ({ t, setTweak }) => (
  <TweaksPanel title="Tweaks">
    <TweakSection title="Preset">
      <TweakRadio value={t.preset} options={[{ value: "mackes", label: "Mackes" }, { value: "hash", label: "#!" }, { value: "daylight", label: "Day" }]}
        onChange={v => setTweak("preset", v)} />
      <div style={{ height: 8 }} />
      <TweakRadio value={t.preset} options={[{ value: "vanilla", label: "Vanilla" }, { value: "node", label: "Node" }]}
        onChange={v => setTweak("preset", v)} />
    </TweakSection>
    <TweakSection title="Density">
      <TweakRadio value={t.density} options={[{ value: "compact", label: "Compact" }, { value: "cozy", label: "Cozy" }, { value: "comfortable", label: "Comfy" }]}
        onChange={v => setTweak("density", v)} />
    </TweakSection>
    <TweakSection title="Chrome">
      <TweakToggle label="XFCE window frame" value={t.showXfceFrame !== false} onChange={v => setTweak("showXfceFrame", v)} />
      <TweakToggle label="Status bar" value={t.showStatusBar} onChange={v => setTweak("showStatusBar", v)} />
    </TweakSection>
    <TweakSection title="Wizard">
      <TweakButton onClick={() => setTweak("showWizard", true)}>Show first-run wizard</TweakButton>
    </TweakSection>
  </TweaksPanel>
);

// Mount
const root = ReactDOM.createRoot(document.getElementById("root"));
root.render(<App />);
