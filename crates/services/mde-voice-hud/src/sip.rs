//! VOIP-28 — pure-Rust SIP signaling for the softphone (NOT PJSIP, per
//! CLAUDE.md §1's pure-Rust lock; operator decision 2026-06-07).
//!
//! Slice 1: a SIP account loaded from `~/.config/mde/voice/account.toml` and a
//! real `REGISTER` over UDP with RFC 2617 / RFC 7616 digest auth. Requests are
//! built as SIP text (the wire protocol is text — simple + byte-testable);
//! responses are parsed with `rsip`, and the digest response is produced by
//! `rsip::services::DigestGenerator` (its own md-5/sha2-backed implementation,
//! so no separate crypto dep). The live registrar round-trip needs a running
//! SIP server → that is the SIP-server bench; everything here that does not
//! touch the socket is unit-tested.

use std::fmt::Write as _;
use std::net::{ToSocketAddrs, UdpSocket};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rsip::headers::auth::{Algorithm, AuthQop, Qop};
use rsip::headers::untyped::ToTypedHeader;
use rsip::headers::Header;
use rsip::services::DigestGenerator;
use rsip::{Method, Uri};

/// A SIP account, the credentials the softphone registers with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SipAccount {
    pub username: String,
    pub password: String,
    pub server_host: String,
    pub server_port: u16,
    pub display_name: String,
    pub expires: u32,
}

/// On-disk shape of `account.toml`.
#[derive(serde::Deserialize)]
struct AccountFile {
    username: String,
    #[serde(default)]
    password: String,
    /// Registrar, as `host` or `host:port`.
    server: String,
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default = "default_expires")]
    expires: u32,
}

fn default_expires() -> u32 {
    3600
}

impl SipAccount {
    /// `~/.config/mde/voice/account.toml` (XDG `config_dir`).
    pub fn config_path() -> std::path::PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from(".config"))
            .join("mde")
            .join("voice")
            .join("account.toml")
    }

    /// Load the account, or `None` when no account file is present (the honest
    /// single-node state → the HUD shows "Not registered").
    pub fn load() -> Option<SipAccount> {
        let text = std::fs::read_to_string(Self::config_path()).ok()?;
        Self::from_toml(&text).ok()
    }

    fn from_toml(text: &str) -> Result<SipAccount, String> {
        let f: AccountFile = toml::from_str(text).map_err(|e| e.to_string())?;
        let (server_host, server_port) = split_host_port(&f.server, 5060);
        if f.username.trim().is_empty() || server_host.is_empty() {
            return Err("account.toml needs a username and a server".to_string());
        }
        let display_name = f
            .display_name
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| f.username.clone());
        Ok(SipAccount {
            username: f.username,
            password: f.password,
            server_host,
            server_port,
            display_name,
            expires: f.expires.max(1),
        })
    }

    /// `user@host` address-of-record.
    fn aor(&self) -> String {
        format!("sip:{}@{}", self.username, self.server_host)
    }

    /// The registrar request-URI (`sip:host`).
    fn registrar_uri(&self) -> String {
        format!("sip:{}", self.server_host)
    }
}

/// Split `host` / `host:port`, defaulting the port.
fn split_host_port(server: &str, default_port: u16) -> (String, u16) {
    match server.rsplit_once(':') {
        Some((h, p)) if !h.is_empty() => match p.parse::<u16>() {
            Ok(port) => (h.to_string(), port),
            Err(_) => (server.to_string(), default_port),
        },
        _ => (server.to_string(), default_port),
    }
}

/// Live registration state shown in the HUD topbar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistrationState {
    /// No `account.toml` — nothing to register.
    NoAccount,
    /// A REGISTER is in flight.
    Registering,
    /// The registrar returned 200 OK.
    Registered { server: String, expires: u32 },
    /// The attempt failed (timeout, rejected, unreachable …).
    Failed(String),
}

impl RegistrationState {
    /// One-line topbar label.
    pub fn label(&self) -> String {
        match self {
            RegistrationState::NoAccount => "Not registered".to_string(),
            RegistrationState::Registering => "Registering…".to_string(),
            RegistrationState::Registered { server, .. } => format!("Registered · {server}"),
            RegistrationState::Failed(_) => "Registration failed".to_string(),
        }
    }

    /// Whether the account is live (drives the presence pip).
    pub const fn is_online(&self) -> bool {
        matches!(self, Self::Registered { .. })
    }
}

/// A parsed `WWW-Authenticate` / `Proxy-Authenticate` digest challenge.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Challenge {
    realm: String,
    nonce: String,
    qop: Option<Qop>,
    algorithm: Algorithm,
    opaque: Option<String>,
    /// 407 (proxy) vs 401 (registrar) — picks the Authorization header name.
    proxy: bool,
}

/// Per-attempt transaction identifiers (Call-ID / tag / branch / CSeq). Kept
/// separate so the builder is a pure function the tests can pin.
#[derive(Debug, Clone)]
struct TxnIds {
    call_id: String,
    from_tag: String,
    branch: String,
    cseq: u32,
}

/// Build a REGISTER request as SIP text. `auth` is an optional
/// `(header_name, value)` for the authorized retry.
fn build_register(
    account: &SipAccount,
    local_host: &str,
    local_port: u16,
    ids: &TxnIds,
    auth: Option<(&str, &str)>,
) -> String {
    let aor = account.aor();
    let contact = format!("sip:{}@{local_host}:{local_port}", account.username);
    let version = env!("CARGO_PKG_VERSION");
    let mut m = String::new();
    let _ = write!(m, "REGISTER {} SIP/2.0\r\n", account.registrar_uri());
    let _ = write!(
        m,
        "Via: SIP/2.0/UDP {local_host}:{local_port};branch={};rport\r\n",
        ids.branch
    );
    m.push_str("Max-Forwards: 70\r\n");
    let _ = write!(m, "From: <{aor}>;tag={}\r\n", ids.from_tag);
    let _ = write!(m, "To: <{aor}>\r\n");
    let _ = write!(m, "Call-ID: {}\r\n", ids.call_id);
    let _ = write!(m, "CSeq: {} REGISTER\r\n", ids.cseq);
    let _ = write!(m, "Contact: <{contact}>\r\n");
    let _ = write!(m, "Expires: {}\r\n", account.expires);
    let _ = write!(m, "User-Agent: Mackes Workstation Voice/{version}\r\n");
    if let Some((name, value)) = auth {
        let _ = write!(m, "{name}: {value}\r\n");
    }
    m.push_str("Content-Length: 0\r\n\r\n");
    m
}

/// Render an `Algorithm` as its SIP token.
fn algorithm_token(a: Algorithm) -> &'static str {
    match a {
        Algorithm::Md5 => "MD5",
        Algorithm::Md5Sess => "MD5-sess",
        Algorithm::Sha256 => "SHA-256",
        Algorithm::Sha256Sess => "SHA-256-sess",
        Algorithm::Sha512 => "SHA-512",
        Algorithm::Sha512Sess => "SHA-512-sess",
    }
}

/// Compute the digest response and render the matching `Authorization` header
/// value. `nc` is the nonce-count for `qop` challenges.
fn authorization_value(
    account: &SipAccount,
    ch: &Challenge,
    cnonce: &str,
    nc: u8,
) -> Result<String, String> {
    let uri = Uri::try_from(account.registrar_uri()).map_err(|e| e.to_string())?;
    let method = Method::Register;
    let qop = match &ch.qop {
        Some(Qop::Auth) => Some(AuthQop::Auth {
            cnonce: cnonce.to_string(),
            nc,
        }),
        Some(Qop::AuthInt) => Some(AuthQop::AuthInt {
            cnonce: cnonce.to_string(),
            nc,
        }),
        None => None,
    };
    let response = DigestGenerator {
        username: &account.username,
        password: &account.password,
        nonce: &ch.nonce,
        uri: &uri,
        realm: &ch.realm,
        method: &method,
        qop: qop.as_ref(),
        algorithm: ch.algorithm,
    }
    .compute();

    let mut v = format!(
        "Digest username=\"{}\", realm=\"{}\", nonce=\"{}\", uri=\"{}\", response=\"{}\", algorithm={}",
        account.username,
        ch.realm,
        ch.nonce,
        account.registrar_uri(),
        response,
        algorithm_token(ch.algorithm),
    );
    if ch.qop.is_some() {
        let qop_tok = match &ch.qop {
            Some(Qop::AuthInt) => "auth-int",
            _ => "auth",
        };
        // nc is formatted to match DigestGenerator (decimal, 8-wide).
        let _ = write!(v, ", qop={qop_tok}, cnonce=\"{cnonce}\", nc={nc:08}");
    }
    if let Some(opaque) = &ch.opaque {
        let _ = write!(v, ", opaque=\"{opaque}\"");
    }
    Ok(v)
}

/// Extract a digest challenge from a 401/407 response.
fn parse_challenge(resp: &rsip::Response) -> Option<Challenge> {
    for h in resp.headers.iter() {
        match h {
            Header::WwwAuthenticate(w) => {
                if let Ok(t) = w.typed() {
                    return Some(Challenge {
                        realm: t.realm,
                        nonce: t.nonce,
                        qop: t.qop,
                        algorithm: t.algorithm.unwrap_or(Algorithm::Md5),
                        opaque: t.opaque,
                        proxy: false,
                    });
                }
            }
            Header::ProxyAuthenticate(p) => {
                if let Ok(t) = p.typed() {
                    // `ProxyAuthenticate` is a newtype around `WwwAuthenticate`.
                    let t = t.0;
                    return Some(Challenge {
                        realm: t.realm,
                        nonce: t.nonce,
                        qop: t.qop,
                        algorithm: t.algorithm.unwrap_or(Algorithm::Md5),
                        opaque: t.opaque,
                        proxy: true,
                    });
                }
            }
            _ => {}
        }
    }
    None
}

/// Read the granted `Expires` from a 200 OK (header, else the Contact param),
/// falling back to the requested value.
fn parse_granted_expires(resp: &rsip::Response, requested: u32) -> u32 {
    for h in resp.headers.iter() {
        if let Header::Expires(e) = h {
            if let Ok(secs) = e.seconds() {
                return secs;
            }
        }
    }
    requested
}

/// A monotonic, collision-free token for Call-ID / tags / branches / cnonce.
fn gen_token(prefix: &str) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let c = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{prefix}{n:x}{c:x}")
}

fn recv_response(sock: &UdpSocket) -> Result<rsip::Response, String> {
    let mut buf = [0u8; 4096];
    let n = sock
        .recv(&mut buf)
        .map_err(|e| format!("no reply from registrar ({e})"))?;
    rsip::Response::try_from(&buf[..n]).map_err(|e| format!("malformed SIP reply ({e})"))
}

/// Attempt a single REGISTER, returning the granted expiry on success.
///
/// Blocking + socket-touching → call off the UI thread (the HUD runs it via
/// `Task::perform`). Never panics: every failure maps to `Err(String)`.
pub fn register_once(account: &SipAccount, timeout: Duration) -> RegistrationState {
    match try_register(account, timeout) {
        Ok(expires) => RegistrationState::Registered {
            server: format!("{}:{}", account.server_host, account.server_port),
            expires,
        },
        Err(e) => RegistrationState::Failed(e),
    }
}

fn try_register(account: &SipAccount, timeout: Duration) -> Result<u32, String> {
    let server_addr = (account.server_host.as_str(), account.server_port)
        .to_socket_addrs()
        .map_err(|e| format!("cannot resolve {}: {e}", account.server_host))?
        .next()
        .ok_or_else(|| format!("no address for {}", account.server_host))?;

    let sock = UdpSocket::bind("0.0.0.0:0").map_err(|e| format!("socket bind failed ({e})"))?;
    sock.set_read_timeout(Some(timeout)).ok();
    sock.connect(server_addr)
        .map_err(|e| format!("connect failed ({e})"))?;
    let local = sock
        .local_addr()
        .map_err(|e| format!("no local addr ({e})"))?;
    let local_host = local.ip().to_string();
    let local_port = local.port();

    let call_id = gen_token("mwv-");
    let from_tag = gen_token("t");
    let ids = TxnIds {
        call_id: call_id.clone(),
        from_tag: from_tag.clone(),
        branch: format!("z9hG4bK{}", gen_token("")),
        cseq: 1,
    };

    // First REGISTER (unauthenticated).
    let req = build_register(account, &local_host, local_port, &ids, None);
    sock.send(req.as_bytes())
        .map_err(|e| format!("send failed ({e})"))?;
    let resp = recv_response(&sock)?;
    let code = u16::from(resp.status_code.clone());

    if code == 200 {
        return Ok(parse_granted_expires(&resp, account.expires));
    }
    if code != 401 && code != 407 {
        return Err(format!("registrar replied {code}"));
    }

    // Authenticated retry.
    let ch = parse_challenge(&resp).ok_or("auth challenge missing or unparseable")?;
    let cnonce = gen_token("c");
    let auth_value = authorization_value(account, &ch, &cnonce, 1)?;
    let header_name = if ch.proxy {
        "Proxy-Authorization"
    } else {
        "Authorization"
    };
    let ids2 = TxnIds {
        call_id,
        from_tag,
        branch: format!("z9hG4bK{}", gen_token("")),
        cseq: 2,
    };
    let req2 = build_register(
        account,
        &local_host,
        local_port,
        &ids2,
        Some((header_name, &auth_value)),
    );
    sock.send(req2.as_bytes())
        .map_err(|e| format!("send failed ({e})"))?;
    let resp2 = recv_response(&sock)?;
    let code2 = u16::from(resp2.status_code.clone());
    if code2 == 200 {
        Ok(parse_granted_expires(&resp2, account.expires))
    } else {
        Err(format!("registrar rejected auth ({code2})"))
    }
}

// ── VOIP-28 slice 2: outbound call signaling (INVITE / SDP / ACK / BYE) ──────
//
// The dialog establishment is real (INVITE → digest auth → 180 Ringing → 200 OK
// → ACK, BYE to hang up) and the SDP answer is parsed into the remote RTP
// endpoint. Media (RTP/G.711 over that endpoint) is slice 3 — until then a
// connected call carries no audio, which the HUD states honestly.

/// Live call state shown in the HUD.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallState {
    /// No call in progress.
    Idle,
    /// INVITE sent, awaiting a final response.
    Calling { peer: String },
    /// 180 Ringing received.
    Ringing { peer: String },
    /// 200 OK + ACK — the dialog is up (audio lands in slice 3).
    InCall { peer: String },
    /// The call ended (local or remote BYE).
    Ended,
    /// Setup failed (busy, declined, timeout, unreachable…).
    Failed(String),
}

impl CallState {
    /// One-line label for the dialer status row.
    pub fn label(&self) -> String {
        match self {
            CallState::Idle => String::new(),
            CallState::Calling { peer } => format!("Calling {peer}…"),
            CallState::Ringing { peer } => format!("Ringing {peer}…"),
            CallState::InCall { peer } => format!("In call · {peer} (no audio yet)"),
            CallState::Ended => "Call ended".to_string(),
            CallState::Failed(why) => format!("Call failed: {why}"),
        }
    }

    /// Whether a call is active (dialog up or being set up).
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            CallState::Calling { .. } | CallState::Ringing { .. } | CallState::InCall { .. }
        )
    }
}

/// Remote media endpoint parsed from the SDP answer (slice 3 sends RTP here).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteMedia {
    pub addr: String,
    pub port: u16,
    /// RTP payload type: 0 = PCMU (G.711 µ-law), 8 = PCMA (G.711 A-law).
    pub payload_type: u8,
}

/// An established dialog — enough to hang up (BYE) and (slice 3) attach media.
#[derive(Debug, Clone)]
pub struct CallSession {
    account: SipAccount,
    target: String,
    call_id: String,
    from_tag: String,
    to_tag: String,
    local_host: String,
    local_port: u16,
    /// The local RTP port advertised in the SDP offer (slice 3 binds it).
    pub rtp_port: u16,
    /// Where the peer wants RTP (slice 3 target).
    pub remote: RemoteMedia,
    cseq: u32,
}

/// Normalize a dialed string into a request-URI. A bare number/extension
/// becomes `sip:<number>@<registrar>`; an already-qualified `sip:` URI or
/// `user@host` is used as given.
fn target_uri(account: &SipAccount, dialed: &str) -> String {
    let d = dialed.trim();
    if d.starts_with("sip:") {
        d.to_string()
    } else if d.contains('@') {
        format!("sip:{d}")
    } else {
        // Strip dial-formatting (spaces, parens, dashes); keep digits + + * #.
        let digits: String = d
            .chars()
            .filter(|c| c.is_ascii_digit() || matches!(c, '+' | '*' | '#'))
            .collect();
        format!("sip:{digits}@{}", account.server_host)
    }
}

/// Minimal audio SDP offer — PCMU(0) + PCMA(8) at 8 kHz on `rtp_port`.
fn build_sdp_offer(local_host: &str, rtp_port: u16) -> String {
    format!(
        "v=0\r\n\
         o=mwv 0 0 IN IP4 {local_host}\r\n\
         s=Mackes Workstation Voice\r\n\
         c=IN IP4 {local_host}\r\n\
         t=0 0\r\n\
         m=audio {rtp_port} RTP/AVP 0 8\r\n\
         a=rtpmap:0 PCMU/8000\r\n\
         a=rtpmap:8 PCMA/8000\r\n\
         a=sendrecv\r\n"
    )
}

/// Parse the connection address + first audio media line from an SDP body.
fn parse_sdp(body: &str) -> Option<RemoteMedia> {
    let mut addr: Option<String> = None;
    let mut port: Option<u16> = None;
    let mut pt: Option<u8> = None;
    for line in body.lines() {
        if let Some(rest) = line.strip_prefix("c=IN IP4 ") {
            addr = Some(rest.trim().to_string());
        } else if let Some(rest) = line.strip_prefix("m=audio ") {
            let mut it = rest.split_whitespace();
            port = it.next().and_then(|p| p.parse::<u16>().ok());
            let _proto = it.next(); // RTP/AVP
            pt = it.next().and_then(|p| p.parse::<u8>().ok());
        }
    }
    Some(RemoteMedia {
        addr: addr?,
        port: port?,
        payload_type: pt.unwrap_or(0),
    })
}

/// Build an INVITE (with SDP offer) or its authorized retry.
fn build_invite(
    account: &SipAccount,
    target: &str,
    local_host: &str,
    local_port: u16,
    ids: &TxnIds,
    sdp: &str,
    auth: Option<(&str, &str)>,
) -> String {
    let from = account.aor();
    let contact = format!("sip:{}@{local_host}:{local_port}", account.username);
    let mut m = String::new();
    let _ = write!(m, "INVITE {target} SIP/2.0\r\n");
    let _ = write!(
        m,
        "Via: SIP/2.0/UDP {local_host}:{local_port};branch={};rport\r\n",
        ids.branch
    );
    m.push_str("Max-Forwards: 70\r\n");
    let _ = write!(m, "From: <{from}>;tag={}\r\n", ids.from_tag);
    let _ = write!(m, "To: <{target}>\r\n");
    let _ = write!(m, "Call-ID: {}\r\n", ids.call_id);
    let _ = write!(m, "CSeq: {} INVITE\r\n", ids.cseq);
    let _ = write!(m, "Contact: <{contact}>\r\n");
    if let Some((name, value)) = auth {
        let _ = write!(m, "{name}: {value}\r\n");
    }
    m.push_str("Content-Type: application/sdp\r\n");
    let _ = write!(m, "Content-Length: {}\r\n\r\n", sdp.len());
    m.push_str(sdp);
    m
}

/// Build the in-dialog ACK for a 2xx (its own transaction, same branch rules).
fn build_ack(session: &CallSession, branch: &str) -> String {
    let from = session.account.aor();
    let mut m = String::new();
    let _ = write!(m, "ACK {} SIP/2.0\r\n", session.target);
    let _ = write!(
        m,
        "Via: SIP/2.0/UDP {}:{};branch={branch};rport\r\n",
        session.local_host, session.local_port
    );
    m.push_str("Max-Forwards: 70\r\n");
    let _ = write!(m, "From: <{from}>;tag={}\r\n", session.from_tag);
    let _ = write!(m, "To: <{}>;tag={}\r\n", session.target, session.to_tag);
    let _ = write!(m, "Call-ID: {}\r\n", session.call_id);
    let _ = write!(m, "CSeq: {} ACK\r\n", session.cseq);
    m.push_str("Content-Length: 0\r\n\r\n");
    m
}

/// Build a BYE to tear down an established dialog.
fn build_bye(session: &CallSession, branch: &str, cseq: u32) -> String {
    let from = session.account.aor();
    let mut m = String::new();
    let _ = write!(m, "BYE {} SIP/2.0\r\n", session.target);
    let _ = write!(
        m,
        "Via: SIP/2.0/UDP {}:{};branch={branch};rport\r\n",
        session.local_host, session.local_port
    );
    m.push_str("Max-Forwards: 70\r\n");
    let _ = write!(m, "From: <{from}>;tag={}\r\n", session.from_tag);
    let _ = write!(m, "To: <{}>;tag={}\r\n", session.target, session.to_tag);
    let _ = write!(m, "Call-ID: {}\r\n", session.call_id);
    let _ = write!(m, "CSeq: {cseq} BYE\r\n");
    m.push_str("Content-Length: 0\r\n\r\n");
    m
}

/// Read the To-tag from a response's To header (needed to address ACK/BYE).
fn parse_to_tag(resp: &rsip::Response) -> Option<String> {
    for h in resp.headers.iter() {
        if let Header::To(t) = h {
            if let Ok(typed) = t.typed() {
                if let Some(tag) = typed.tag() {
                    return Some(tag.to_string());
                }
            }
        }
    }
    None
}

/// Place an outbound call: INVITE (+ digest retry) → await a final response,
/// ACK a 2xx, and return the established `CallSession`. Blocking + socket —
/// run off the UI thread. The live audio path is slice 3 (RTP/ALSA).
pub fn place_call(
    account: &SipAccount,
    dialed: &str,
    ring_timeout: Duration,
) -> Result<CallSession, String> {
    let target = target_uri(account, dialed);
    let server_addr = (account.server_host.as_str(), account.server_port)
        .to_socket_addrs()
        .map_err(|e| format!("cannot resolve {}: {e}", account.server_host))?
        .next()
        .ok_or_else(|| format!("no address for {}", account.server_host))?;
    let sock = UdpSocket::bind("0.0.0.0:0").map_err(|e| format!("socket bind failed ({e})"))?;
    sock.set_read_timeout(Some(Duration::from_secs(2))).ok();
    sock.connect(server_addr)
        .map_err(|e| format!("connect failed ({e})"))?;
    let local = sock
        .local_addr()
        .map_err(|e| format!("no local addr ({e})"))?;
    let local_host = local.ip().to_string();
    let local_port = local.port();
    // Advertise an RTP port (slice 3 binds it); derive it from the signaling
    // port range so it is deterministic per call without a second bind here.
    let rtp_port = 40000 + (local_port % 1000) * 2;
    let sdp = build_sdp_offer(&local_host, rtp_port);

    let call_id = gen_token("mwv-");
    let from_tag = gen_token("t");
    let mut ids = TxnIds {
        call_id: call_id.clone(),
        from_tag: from_tag.clone(),
        branch: format!("z9hG4bK{}", gen_token("")),
        cseq: 1,
    };

    let req = build_invite(account, &target, &local_host, local_port, &ids, &sdp, None);
    sock.send(req.as_bytes())
        .map_err(|e| format!("send failed ({e})"))?;

    // Await a final (>=200) response, honouring provisional 1xx and one auth
    // challenge, bounded by ring_timeout.
    let deadline_passes = (ring_timeout.as_secs().max(1) / 2 + 1) as u32 * 8;
    let mut authed = false;
    for _ in 0..deadline_passes {
        let resp = match recv_response(&sock) {
            Ok(r) => r,
            Err(_) => continue, // 2s read timeout tick; keep waiting for ring_timeout
        };
        let code = u16::from(resp.status_code.clone());
        match code {
            100..=199 => continue, // Trying / Ringing — keep waiting
            200..=299 => {
                let to_tag = parse_to_tag(&resp).unwrap_or_default();
                let remote = parse_sdp(&String::from_utf8_lossy(resp.body()))
                    .ok_or("200 OK without a usable SDP answer")?;
                let session = CallSession {
                    account: account.clone(),
                    target: target.clone(),
                    call_id,
                    from_tag,
                    to_tag,
                    local_host,
                    local_port,
                    rtp_port,
                    remote,
                    cseq: ids.cseq,
                };
                let ack = build_ack(&session, &format!("z9hG4bK{}", gen_token("")));
                sock.send(ack.as_bytes())
                    .map_err(|e| format!("ACK send failed ({e})"))?;
                return Ok(session);
            }
            401 | 407 if !authed => {
                let ch = parse_challenge(&resp).ok_or("auth challenge unparseable")?;
                // ACK the failure response (INVITE 4xx requires an ACK).
                authed = true;
                let auth_value = authorization_value(account, &ch, &gen_token("c"), 1)?;
                let name = if ch.proxy {
                    "Proxy-Authorization"
                } else {
                    "Authorization"
                };
                ids = TxnIds {
                    call_id: ids.call_id.clone(),
                    from_tag: ids.from_tag.clone(),
                    branch: format!("z9hG4bK{}", gen_token("")),
                    cseq: 2,
                };
                let req2 = build_invite(
                    account,
                    &target,
                    &local_host,
                    local_port,
                    &ids,
                    &sdp,
                    Some((name, &auth_value)),
                );
                sock.send(req2.as_bytes())
                    .map_err(|e| format!("send failed ({e})"))?;
            }
            486 => return Err("busy".to_string()),
            603 => return Err("declined".to_string()),
            other => return Err(format!("call rejected ({other})")),
        }
    }
    Err("no answer (timeout)".to_string())
}

/// Tear down an established call with a BYE (best-effort; never panics).
pub fn hang_up(session: &CallSession) -> Result<(), String> {
    let server_addr = (
        session.account.server_host.as_str(),
        session.account.server_port,
    )
        .to_socket_addrs()
        .map_err(|e| e.to_string())?
        .next()
        .ok_or("no address")?;
    let sock = UdpSocket::bind("0.0.0.0:0").map_err(|e| e.to_string())?;
    sock.set_read_timeout(Some(Duration::from_secs(1))).ok();
    sock.connect(server_addr).map_err(|e| e.to_string())?;
    let bye = build_bye(
        session,
        &format!("z9hG4bK{}", gen_token("")),
        session.cseq + 1,
    );
    sock.send(bye.as_bytes()).map_err(|e| e.to_string())?;
    let _ = recv_response(&sock); // best-effort 200 OK
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_account() -> SipAccount {
        SipAccount {
            username: "alice".into(),
            password: "secret".into(),
            server_host: "sip.example.com".into(),
            server_port: 5060,
            display_name: "Alice".into(),
            expires: 3600,
        }
    }

    #[test]
    fn split_host_port_defaults_and_explicit() {
        assert_eq!(split_host_port("host", 5060), ("host".into(), 5060));
        assert_eq!(split_host_port("host:5080", 5060), ("host".into(), 5080));
        // A bare IPv4 with no port keeps the default.
        assert_eq!(split_host_port("10.0.0.1", 5060), ("10.0.0.1".into(), 5060));
    }

    #[test]
    fn from_toml_parses_minimal_account() {
        let a = SipAccount::from_toml(
            "username = \"alice\"\npassword = \"secret\"\nserver = \"sip.example.com:5080\"\n",
        )
        .unwrap();
        assert_eq!(a.username, "alice");
        assert_eq!(a.server_host, "sip.example.com");
        assert_eq!(a.server_port, 5080);
        assert_eq!(a.display_name, "alice"); // defaults to username
        assert_eq!(a.expires, 3600);
    }

    #[test]
    fn from_toml_rejects_empty_username() {
        assert!(SipAccount::from_toml("username = \"\"\nserver = \"h\"\n").is_err());
    }

    #[test]
    fn build_register_has_required_lines() {
        let ids = TxnIds {
            call_id: "cid1".into(),
            from_tag: "tag1".into(),
            branch: "z9hG4bKbranch1".into(),
            cseq: 1,
        };
        let msg = build_register(&sample_account(), "192.168.1.5", 5062, &ids, None);
        assert!(msg.starts_with("REGISTER sip:sip.example.com SIP/2.0\r\n"));
        assert!(msg.contains("Via: SIP/2.0/UDP 192.168.1.5:5062;branch=z9hG4bKbranch1;rport\r\n"));
        assert!(msg.contains("From: <sip:alice@sip.example.com>;tag=tag1\r\n"));
        assert!(msg.contains("To: <sip:alice@sip.example.com>\r\n"));
        assert!(msg.contains("Call-ID: cid1\r\n"));
        assert!(msg.contains("CSeq: 1 REGISTER\r\n"));
        assert!(msg.contains("Contact: <sip:alice@192.168.1.5:5062>\r\n"));
        assert!(msg.contains("Expires: 3600\r\n"));
        assert!(msg.contains("Content-Length: 0\r\n"));
        assert!(msg.ends_with("\r\n\r\n"));
        // No auth header on the first pass.
        assert!(!msg.contains("Authorization:"));
    }

    #[test]
    fn build_register_embeds_auth_when_given() {
        let ids = TxnIds {
            call_id: "cid".into(),
            from_tag: "t".into(),
            branch: "z9hG4bKb".into(),
            cseq: 2,
        };
        let msg = build_register(
            &sample_account(),
            "10.0.0.2",
            5060,
            &ids,
            Some(("Authorization", "Digest realm=\"r\"")),
        );
        assert!(msg.contains("Authorization: Digest realm=\"r\"\r\n"));
        assert!(msg.contains("CSeq: 2 REGISTER\r\n"));
    }

    #[test]
    fn authorization_value_no_qop_matches_digest_generator() {
        // RFC 2617-style inputs, qop absent.
        let acct = SipAccount {
            username: "Mufasa".into(),
            password: "Circle Of Life".into(),
            server_host: "host.com".into(),
            server_port: 5060,
            display_name: "Mufasa".into(),
            expires: 60,
        };
        let ch = Challenge {
            realm: "testrealm@host.com".into(),
            nonce: "dcd98b7102dd2f0e8b11d0f600bfb0c093".into(),
            qop: None,
            algorithm: Algorithm::Md5,
            opaque: None,
            proxy: false,
        };
        let value = authorization_value(&acct, &ch, "unused", 1).unwrap();
        // Independently compute the expected response via the same generator.
        let uri = Uri::try_from(acct.registrar_uri()).unwrap();
        let method = Method::Register;
        let expected = DigestGenerator {
            username: &acct.username,
            password: &acct.password,
            nonce: &ch.nonce,
            uri: &uri,
            realm: &ch.realm,
            method: &method,
            qop: None,
            algorithm: Algorithm::Md5,
        }
        .compute();
        assert!(value.contains(&format!("response=\"{expected}\"")));
        assert!(value.contains("username=\"Mufasa\""));
        assert!(value.contains("realm=\"testrealm@host.com\""));
        assert!(value.contains("algorithm=MD5"));
        // No qop machinery when the challenge omits it.
        assert!(!value.contains("qop="));
        assert!(!value.contains("cnonce="));
    }

    #[test]
    fn authorization_value_qop_auth_includes_cnonce_nc() {
        let ch = Challenge {
            realm: "r".into(),
            nonce: "n".into(),
            qop: Some(Qop::Auth),
            algorithm: Algorithm::Md5,
            opaque: Some("op".into()),
            proxy: false,
        };
        let value = authorization_value(&sample_account(), &ch, "abc123", 1).unwrap();
        assert!(value.contains("qop=auth"));
        assert!(value.contains("cnonce=\"abc123\""));
        assert!(value.contains("nc=00000001"));
        assert!(value.contains("opaque=\"op\""));
    }

    #[test]
    fn parse_challenge_reads_www_authenticate() {
        let raw = "SIP/2.0 401 Unauthorized\r\n\
             Via: SIP/2.0/UDP 10.0.0.2:5060;branch=z9hG4bKx\r\n\
             From: <sip:alice@sip.example.com>;tag=t\r\n\
             To: <sip:alice@sip.example.com>;tag=s\r\n\
             Call-ID: cid\r\n\
             CSeq: 1 REGISTER\r\n\
             WWW-Authenticate: Digest realm=\"asterisk\", nonce=\"abc\", algorithm=MD5, qop=\"auth\"\r\n\
             Content-Length: 0\r\n\r\n";
        let resp = rsip::Response::try_from(raw.as_bytes()).unwrap();
        let ch = parse_challenge(&resp).expect("challenge");
        assert_eq!(ch.realm, "asterisk");
        assert_eq!(ch.nonce, "abc");
        assert!(!ch.proxy);
        assert!(matches!(ch.qop, Some(Qop::Auth)));
    }

    #[test]
    fn registration_state_labels() {
        assert_eq!(RegistrationState::NoAccount.label(), "Not registered");
        assert_eq!(RegistrationState::Registering.label(), "Registering…");
        assert_eq!(
            RegistrationState::Registered {
                server: "sip.example.com:5060".into(),
                expires: 3600
            }
            .label(),
            "Registered · sip.example.com:5060"
        );
        assert!(RegistrationState::Registered {
            server: "h".into(),
            expires: 1
        }
        .is_online());
        assert!(!RegistrationState::Failed("x".into()).is_online());
    }

    // ── slice 2: call signaling ──────────────────────────────────────────

    #[test]
    fn target_uri_normalizes_dialed_strings() {
        let a = sample_account();
        assert_eq!(target_uri(&a, "1001"), "sip:1001@sip.example.com");
        assert_eq!(
            target_uri(&a, "(415) 555 1234"),
            "sip:4155551234@sip.example.com"
        );
        assert_eq!(target_uri(&a, "bob@other.net"), "sip:bob@other.net");
        assert_eq!(target_uri(&a, "sip:bob@other.net"), "sip:bob@other.net");
    }

    #[test]
    fn sdp_offer_advertises_g711_audio() {
        let sdp = build_sdp_offer("10.0.0.5", 40002);
        assert!(sdp.contains("m=audio 40002 RTP/AVP 0 8\r\n"));
        assert!(sdp.contains("a=rtpmap:0 PCMU/8000\r\n"));
        assert!(sdp.contains("a=rtpmap:8 PCMA/8000\r\n"));
        assert!(sdp.contains("c=IN IP4 10.0.0.5\r\n"));
    }

    #[test]
    fn parse_sdp_extracts_remote_endpoint() {
        let body = "v=0\r\no=x 0 0 IN IP4 1.2.3.4\r\nc=IN IP4 1.2.3.4\r\n\
                    t=0 0\r\nm=audio 5004 RTP/AVP 8 0\r\na=rtpmap:8 PCMA/8000\r\n";
        let r = parse_sdp(body).expect("sdp");
        assert_eq!(r.addr, "1.2.3.4");
        assert_eq!(r.port, 5004);
        assert_eq!(r.payload_type, 8);
    }

    #[test]
    fn build_invite_carries_sdp_body_and_length() {
        let ids = TxnIds {
            call_id: "cid".into(),
            from_tag: "ft".into(),
            branch: "z9hG4bKb".into(),
            cseq: 1,
        };
        let sdp = build_sdp_offer("10.0.0.5", 40002);
        let msg = build_invite(
            &sample_account(),
            "sip:1001@sip.example.com",
            "10.0.0.5",
            5070,
            &ids,
            &sdp,
            None,
        );
        assert!(msg.starts_with("INVITE sip:1001@sip.example.com SIP/2.0\r\n"));
        assert!(msg.contains("CSeq: 1 INVITE\r\n"));
        assert!(msg.contains("Content-Type: application/sdp\r\n"));
        assert!(msg.contains(&format!("Content-Length: {}\r\n", sdp.len())));
        assert!(msg.ends_with(&sdp));
    }

    fn sample_session() -> CallSession {
        CallSession {
            account: sample_account(),
            target: "sip:1001@sip.example.com".into(),
            call_id: "cid".into(),
            from_tag: "ft".into(),
            to_tag: "tt".into(),
            local_host: "10.0.0.5".into(),
            local_port: 5070,
            rtp_port: 40002,
            remote: RemoteMedia {
                addr: "1.2.3.4".into(),
                port: 5004,
                payload_type: 0,
            },
            cseq: 1,
        }
    }

    #[test]
    fn ack_and_bye_address_the_established_dialog() {
        let s = sample_session();
        let ack = build_ack(&s, "z9hG4bKack");
        assert!(ack.starts_with("ACK sip:1001@sip.example.com SIP/2.0\r\n"));
        assert!(ack.contains("To: <sip:1001@sip.example.com>;tag=tt\r\n"));
        assert!(ack.contains("From: <sip:alice@sip.example.com>;tag=ft\r\n"));
        assert!(ack.contains("Call-ID: cid\r\n"));
        assert!(ack.contains("CSeq: 1 ACK\r\n"));

        let bye = build_bye(&s, "z9hG4bKbye", 2);
        assert!(bye.starts_with("BYE sip:1001@sip.example.com SIP/2.0\r\n"));
        assert!(bye.contains("To: <sip:1001@sip.example.com>;tag=tt\r\n"));
        assert!(bye.contains("CSeq: 2 BYE\r\n"));
    }

    #[test]
    fn call_state_labels_and_active() {
        assert_eq!(CallState::Idle.label(), "");
        assert_eq!(
            CallState::Ringing {
                peer: "1001".into()
            }
            .label(),
            "Ringing 1001…"
        );
        assert!(CallState::InCall { peer: "x".into() }.is_active());
        assert!(CallState::Calling { peer: "x".into() }.is_active());
        assert!(!CallState::Idle.is_active());
        assert!(!CallState::Ended.is_active());
    }
}
