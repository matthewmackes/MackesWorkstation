/* Shared Carbon-style atoms */
const { useState, useEffect, useRef, useMemo, useCallback } = React;

// --- Icons (inline SVG, Carbon-ish) ---
const Icon = ({ name, size = 16, color }) => {
  const sw = 1.5;
  const common = { width: size, height: size, viewBox: "0 0 16 16", fill: "none", stroke: color || "currentColor", strokeWidth: sw, strokeLinecap: "square" };
  const paths = {
    dashboard: <><rect x="1" y="1" width="6" height="6"/><rect x="9" y="1" width="6" height="6"/><rect x="1" y="9" width="6" height="6"/><rect x="9" y="9" width="6" height="6"/></>,
    paint: <><path d="M2 14 L8 8 L12 12 L6 14 Z"/><path d="M9 7 L14 2"/></>,
    devices: <><rect x="2" y="3" width="12" height="8"/><path d="M5 14 L11 14"/><path d="M8 11 L8 14"/></>,
    network: <><circle cx="8" cy="8" r="6"/><path d="M2 8 L14 8"/><path d="M8 2 C5 5 5 11 8 14"/><path d="M8 2 C11 5 11 11 8 14"/></>,
    mesh: <><circle cx="3" cy="3" r="2"/><circle cx="13" cy="3" r="2"/><circle cx="3" cy="13" r="2"/><circle cx="13" cy="13" r="2"/><circle cx="8" cy="8" r="2"/><path d="M5 3 L11 3 M3 5 L3 11 M13 5 L13 11 M5 13 L11 13 M4.5 4.5 L6.5 6.5 M11.5 4.5 L9.5 6.5 M4.5 11.5 L6.5 9.5 M11.5 11.5 L9.5 9.5"/></>,
    system: <><path d="M3 3 L13 3 L13 13 L3 13 Z"/><path d="M3 6 L13 6"/><circle cx="5" cy="4.5" r="0.5" fill="currentColor"/></>,
    apps: <><rect x="1" y="1" width="6" height="6"/><rect x="9" y="1" width="6" height="6"/><rect x="1" y="9" width="6" height="6"/><circle cx="12" cy="12" r="3"/></>,
    wrench: <><path d="M11 2 L14 5 L11 8 L8 5 Z"/><path d="M9 6 L2 13 L3 14 L10 7"/></>,
    help: <><circle cx="8" cy="8" r="6"/><path d="M6 6 C6 4 10 4 10 6 C10 8 8 7 8 10"/><circle cx="8" cy="13" r="0.5" fill="currentColor"/></>,
    search: <><circle cx="6.5" cy="6.5" r="4.5"/><path d="M10 10 L14 14"/></>,
    bell: <><path d="M3 11 L13 11 L12 6 C12 4 10 2 8 2 C6 2 4 4 4 6 Z"/><path d="M7 13 C7 14 9 14 9 13"/></>,
    user: <><circle cx="8" cy="5" r="3"/><path d="M2 14 C2 11 14 11 14 14"/></>,
    plus: <><path d="M8 2 L8 14 M2 8 L14 8"/></>,
    refresh: <><path d="M14 8 A6 6 0 1 1 12 3.5 L14 5.5 L11 5.5"/></>,
    chevron: <><path d="M4 6 L8 10 L12 6"/></>,
    right: <><path d="M6 4 L10 8 L6 12"/></>,
    check: <><path d="M3 8 L7 12 L13 4"/></>,
    close: <><path d="M3 3 L13 13 M13 3 L3 13"/></>,
    download: <><path d="M8 2 L8 11 M4 8 L8 12 L12 8"/><path d="M2 14 L14 14"/></>,
    trash: <><path d="M3 4 L13 4 M6 4 L6 2 L10 2 L10 4 M5 4 L5 14 L11 14 L11 4"/></>,
    play: <><path d="M5 3 L13 8 L5 13 Z" fill="currentColor"/></>,
    pause: <><rect x="4" y="3" width="3" height="10"/><rect x="9" y="3" width="3" height="10"/></>,
    settings: <><circle cx="8" cy="8" r="2"/><path d="M8 1 L8 3 M8 13 L8 15 M1 8 L3 8 M13 8 L15 8 M3 3 L4.5 4.5 M11.5 11.5 L13 13 M3 13 L4.5 11.5 M11.5 4.5 L13 3"/></>,
    folder: <><path d="M1 4 L6 4 L7 6 L15 6 L15 13 L1 13 Z"/></>,
    cube: <><path d="M8 2 L14 5 L14 11 L8 14 L2 11 L2 5 Z"/><path d="M2 5 L8 8 L14 5 M8 8 L8 14"/></>,
    bolt: <><path d="M9 1 L4 9 L8 9 L7 15 L12 7 L8 7 Z" fill="currentColor"/></>,
    server: <><rect x="2" y="2" width="12" height="5"/><rect x="2" y="9" width="12" height="5"/><circle cx="5" cy="4.5" r="0.5" fill="currentColor"/><circle cx="5" cy="11.5" r="0.5" fill="currentColor"/></>,
    laptop: <><rect x="2" y="3" width="12" height="8"/><path d="M1 13 L15 13"/></>,
    phone: <><rect x="4" y="1" width="8" height="14"/><circle cx="8" cy="13" r="0.5" fill="currentColor"/></>,
    nas: <><rect x="2" y="3" width="12" height="10"/><path d="M2 6 L14 6 M2 9 L14 9"/><circle cx="12" cy="4.5" r="0.5" fill="currentColor"/></>,
    cloud: <><path d="M4 12 C2 12 1 10 2 8 C2 6 4 5 6 6 C6 4 8 3 10 4 C12 4 14 6 13 8 C14 8 15 10 14 12 Z"/></>,
    image: <><rect x="1" y="2" width="14" height="12"/><circle cx="5" cy="6" r="1"/><path d="M1 11 L5 7 L9 11 L11 9 L15 13"/></>,
    monitor: <><rect x="1" y="2" width="14" height="9"/><path d="M5 14 L11 14 M8 11 L8 14"/></>,
    snapshot: <><rect x="1" y="3" width="14" height="11"/><circle cx="8" cy="8.5" r="3"/><rect x="6" y="1" width="4" height="2"/></>,
    drift: <><path d="M2 4 L14 4 M2 8 L14 8 M2 12 L14 12"/><circle cx="6" cy="4" r="1.2" fill="currentColor"/><circle cx="10" cy="8" r="1.2" fill="currentColor"/><circle cx="7" cy="12" r="1.2" fill="currentColor"/></>,
    health: <><path d="M2 8 L5 8 L6 5 L8 11 L9 8 L12 8 L14 8"/></>,
    log: <><path d="M3 1 L11 1 L13 3 L13 15 L3 15 Z"/><path d="M5 6 L11 6 M5 9 L11 9 M5 12 L9 12"/></>,
    reset: <><path d="M14 8 A6 6 0 1 1 12 3.5"/><path d="M14 2 L14 6 L10 6"/></>,
    home: <><path d="M2 7 L8 2 L14 7 L14 14 L2 14 Z"/><path d="M6 14 L6 9 L10 9 L10 14"/></>,
    wifi: <><path d="M1 5 C4 2 12 2 15 5"/><path d="M3 8 C5 6 11 6 13 8"/><path d="M5 11 C6.5 9.5 9.5 9.5 11 11"/><circle cx="8" cy="13" r="0.7" fill="currentColor"/></>,
    lock: <><rect x="3" y="7" width="10" height="7"/><path d="M5 7 L5 5 C5 2 11 2 11 5 L11 7"/></>,
    flame: <><path d="M8 1 C8 4 12 5 12 9 C12 13 4 13 4 9 C4 7 6 6 6 4 C6 6 8 6 8 1 Z"/></>,
    eye: <><path d="M1 8 C3 4 13 4 15 8 C13 12 3 12 1 8 Z"/><circle cx="8" cy="8" r="2"/></>,
    copy: <><rect x="4" y="4" width="9" height="11"/><path d="M3 11 L1 11 L1 1 L9 1 L9 3"/></>,
    keyboard: <><rect x="1" y="4" width="14" height="9"/><path d="M3 7 L4 7 M6 7 L7 7 M9 7 L10 7 M12 7 L13 7 M3 10 L4 10 M5 10 L11 10 M12 10 L13 10"/></>,
    mouse: <><rect x="4" y="1" width="8" height="14" rx="4"/><path d="M8 4 L8 7"/></>,
    sound: <><path d="M2 6 L5 6 L9 3 L9 13 L5 10 L2 10 Z"/><path d="M11 6 C12 7 12 9 11 10"/></>,
    power: <><path d="M5 4 C2 6 2 11 5 13 C8 15 12 13 13 10 C14 7 12 4 9 3"/><path d="M8 1 L8 7"/></>
  };
  return <svg {...common}>{paths[name]}</svg>;
};

// --- Buttons ---
const Btn = ({ kind = "primary", size, icon, children, onClick, disabled, title }) => (
  <button className={`btn ${kind} ${size || ""}`} onClick={onClick} disabled={disabled} title={title}>
    {icon && <Icon name={icon} />}
    {children}
  </button>
);

// --- Tile ---
const Tile = ({ children, clickable, outlined, onClick, style }) => (
  <div className={`tile ${clickable ? "clickable" : ""} ${outlined ? "outlined" : ""}`} onClick={onClick} style={style}>
    {children}
  </div>
);

// --- Tag ---
const Tag = ({ kind = "neutral", children }) => (
  <span className={`tag ${kind}`}>{children}</span>
);

// --- Notification ---
const Notif = ({ kind = "info", title, children, icon }) => (
  <div className={`notif ${kind}`}>
    <div className="notif-icon">{icon || (kind === "success" ? "✓" : kind === "error" ? "✕" : kind === "warning" ? "!" : "i")}</div>
    <div>
      {title && <div className="notif-title">{title}</div>}
      <div className="notif-body">{children}</div>
    </div>
    <div />
  </div>
);

// --- Section header ---
const SectionH = ({ title, meta, children }) => (
  <div className="section-h">
    <h2>{title}</h2>
    <div className="section-meta">{meta || children}</div>
  </div>
);

// --- Status dot ---
const Dot = ({ status = "ok" }) => <span className={`dot-inline ${status}`} />;

// --- Form bits ---
const Field = ({ label, helper, children }) => (
  <div className="form-row">
    <label className="form-label">{label}</label>
    {children}
    {helper && <div className="form-helper">{helper}</div>}
  </div>
);

const Select = ({ value, onChange, options }) => (
  <select className="select" value={value} onChange={e => onChange(e.target.value)}>
    {options.map(o => <option key={o.value || o} value={o.value || o}>{o.label || o}</option>)}
  </select>
);

const Toggle = ({ value, onChange }) => (
  <div className="toggle" data-on={value} onClick={() => onChange(!value)} />
);

// --- DataTable ---
const DataTable = ({ columns, rows, searchable, toolbar, onRowClick, selectedKey, rowKey = "id" }) => {
  const [q, setQ] = useState("");
  const filtered = q ? rows.filter(r => Object.values(r).some(v => String(v).toLowerCase().includes(q.toLowerCase()))) : rows;
  return (
    <div className="dt">
      {(searchable || toolbar) && (
        <div className="dt-toolbar">
          {searchable && (
            <>
              <Icon name="search" />
              <input className="dt-search" placeholder="Search peers, services…" value={q} onChange={e => setQ(e.target.value)} />
            </>
          )}
          <div className="dt-actions">{toolbar}</div>
        </div>
      )}
      <table className="dt-table">
        <thead>
          <tr>{columns.map(c => <th key={c.key} style={{ width: c.width }}>{c.title}</th>)}</tr>
        </thead>
        <tbody>
          {filtered.map(r => (
            <tr key={r[rowKey]} data-selected={selectedKey === r[rowKey]} onClick={() => onRowClick && onRowClick(r)}>
              {columns.map(c => <td key={c.key}>{c.render ? c.render(r) : r[c.key]}</td>)}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
};

// --- Modal ---
const Modal = ({ title, sub, children, onClose, actions }) => (
  <div className="modal-scrim" onClick={onClose}>
    <div className="modal" onClick={e => e.stopPropagation()}>
      <div className="modal-head">
        <h3>{title}</h3>
        {sub && <div className="modal-sub">{sub}</div>}
      </div>
      <div className="modal-body">{children}</div>
      <div className="modal-foot">{actions}</div>
    </div>
  </div>
);

// --- Toast host ---
const ToastHost = ({ toasts }) => (
  <div className="toast-host">
    {toasts.map(t => (
      <div key={t.id} className={`toast ${t.kind || ""}`}>
        <Icon name={t.kind === "success" ? "check" : "bell"} />
        <div>{t.message}</div>
      </div>
    ))}
  </div>
);

Object.assign(window, {
  Icon, Btn, Tile, Tag, Notif, SectionH, Dot, Field, Select, Toggle, DataTable, Modal, ToastHost,
  useState, useEffect, useRef, useMemo, useCallback,
});
