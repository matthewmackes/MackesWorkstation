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
}
