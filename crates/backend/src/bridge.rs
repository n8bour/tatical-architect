//! Persistent TCP line-protocol client for the in-game FM27 BepInEx bridge (`127.0.0.1:27029`).
//!
//! Protocol: one `\n`-delimited line per request, one `\n`-delimited line per reply.
//! Local loopback (`127.0.0.1`) only. Automatically reconnects once if a connection drops.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

/// Default loopback TCP port used by `TacticalBibleBridge.dll`.
pub const BRIDGE_PORT: u16 = 27029;

/// Maximum single-command byte length allowed on the line protocol.
pub const MAX_CMD_LEN: usize = 2_000_000;

const CONNECT_TIMEOUT: Duration = Duration::from_millis(600);
const WRITE_TIMEOUT: Duration = Duration::from_secs(5);

/// Validates that a bridge command is single-line and within the 2 MB protocol cap.
pub fn validate_cmd(cmd: &str) -> Result<(), String> {
    if cmd.contains('\n') || cmd.contains('\r') || cmd.len() > MAX_CMD_LEN {
        return Err("Comando inválido".into());
    }
    Ok(())
}

/// Stateful, thread-safe persistent TCP client for the FM27 Bridge.
pub struct BridgeClient {
    addr: SocketAddr,
    conn: Mutex<Option<TcpStream>>,
}

impl BridgeClient {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            conn: Mutex::new(None),
        }
    }

    pub fn loopback(port: u16) -> Self {
        Self::new(SocketAddr::from(([127, 0, 0, 1], port)))
    }

    fn connect(&self) -> Option<TcpStream> {
        let s = TcpStream::connect_timeout(&self.addr, CONNECT_TIMEOUT).ok()?;
        let _ = s.set_nodelay(true);
        Some(s)
    }

    fn roundtrip(stream: &mut TcpStream, cmd: &str, timeout_ms: u64) -> Result<String, String> {
        let _ = stream.set_read_timeout(Some(Duration::from_millis(timeout_ms)));
        let _ = stream.set_write_timeout(Some(WRITE_TIMEOUT));
        stream.write_all(cmd.as_bytes()).map_err(|e| e.to_string())?;
        stream.write_all(b"\n").map_err(|e| e.to_string())?;

        let mut buf: Vec<u8> = Vec::new();
        let mut chunk = [0u8; 4096];
        loop {
            let n = stream.read(&mut chunk).map_err(|e| e.to_string())?;
            if n == 0 {
                return Err("closed".into());
            }
            buf.extend_from_slice(&chunk[..n]);
            if buf.ends_with(b"\n") {
                break;
            }
        }
        Ok(String::from_utf8_lossy(&buf)
            .trim_end_matches(['\r', '\n'])
            .to_string())
    }

    /// Sends one command line and returns the response line (without trailing `\r\n`).
    /// Reconnects once automatically if a persistent socket was closed by the game.
    pub fn call(&self, cmd: &str, timeout_ms: u64) -> Result<String, String> {
        validate_cmd(cmd)?;
        let mut guard = self.conn.lock().map_err(|e| e.to_string())?;
        for attempt in 0..2 {
            if guard.is_none() {
                *guard = self.connect();
                if guard.is_none() {
                    return Err(
                        "O Bridge não responde (FM27 aberto com o plugin instalado?)".into(),
                    );
                }
            }
            match Self::roundtrip(guard.as_mut().unwrap(), cmd, timeout_ms) {
                Ok(reply) => return Ok(reply),
                Err(e) => {
                    *guard = None;
                    if attempt == 1 {
                        return Err(format!("O Bridge não respondeu: {}", e));
                    }
                }
            }
        }
        Err("O Bridge não respondeu".into())
    }

    /// Drops the cached TCP connection (if any).
    pub fn disconnect(&self) {
        if let Ok(mut guard) = self.conn.lock() {
            *guard = None;
        }
    }
}

static DEFAULT_CLIENT: OnceLock<BridgeClient> = OnceLock::new();

/// Returns the process-wide default client connected to `127.0.0.1:27029`.
pub fn default_client() -> &'static BridgeClient {
    DEFAULT_CLIENT.get_or_init(|| BridgeClient::loopback(BRIDGE_PORT))
}

/// Sends a command to the default `127.0.0.1:27029` FM27 bridge.
pub fn bridge_call(cmd: &str, timeout_ms: Option<u64>) -> Result<String, String> {
    default_client().call(cmd, timeout_ms.unwrap_or(15_000))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader};
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn validate_cmd_rejects_newlines_and_oversize() {
        assert!(validate_cmd("ping").is_ok());
        assert!(validate_cmd("bad\ncmd").is_err());
        assert!(validate_cmd("bad\rcmd").is_err());
        let huge = "a".repeat(MAX_CMD_LEN + 1);
        assert!(validate_cmd(&huge).is_err());
    }

    #[test]
    fn bridge_client_framing_and_reconnect_on_eof() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let server = thread::spawn(move || {
            // Connection 1: answer 1 request, then close to simulate dropped socket.
            let (mut s1, _) = listener.accept().unwrap();
            let mut r1 = BufReader::new(s1.try_clone().unwrap());
            let mut line = String::new();
            r1.read_line(&mut line).unwrap();
            assert_eq!(line, "PING\n");
            s1.write_all(b"{\"ok\":true,\"pong\":1}\r\n").unwrap();
            drop(s1);

            // Connection 2: client auto-reconnects seamlessly.
            let (mut s2, _) = listener.accept().unwrap();
            let mut r2 = BufReader::new(s2.try_clone().unwrap());
            let mut line2 = String::new();
            r2.read_line(&mut line2).unwrap();
            assert_eq!(line2, "VERSION\n");
            s2.write_all(b"{\"version\":\"0.4.2\"}\n").unwrap();
        });

        let client = BridgeClient::new(addr);
        let r1 = client.call("PING", 2000).unwrap();
        assert_eq!(r1, r#"{"ok":true,"pong":1}"#);

        // Give the server a moment to drop s1 before second call exercises reconnect.
        thread::sleep(Duration::from_millis(20));
        let r2 = client.call("VERSION", 2000).unwrap();
        assert_eq!(r2, r#"{"version":"0.4.2"}"#);

        server.join().unwrap();
    }
}
