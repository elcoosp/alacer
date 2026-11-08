//! The inspector's WebSocket server: RFC 6455 over `std::net`, no async
//! runtime, no heavy deps — a devtools server must never weigh on the app it
//! observes.
//!
//! Two resources:
//!
//! - `GET /` — the embedded single-file HTML client (this file's sibling,
//!   `inspector.html`), served with `Connection: close`.
//! - `GET /ws` — the WebSocket: a `hello` greeting, then the event stream
//!   (`docs/inspector-protocol.md` is normative).
//!
//! The stream is push, not poll: the recorder's handle serializes each
//! observation into the broadcast queue as it happens, and the session
//! thread drains it over the socket.

use crate::recorder::CausalRecorder;
use crate::wire::{self, ClientMessage, Hello, PROTOCOL_VERSION, WireEvent};
use base64::Engine as _;
use sha1::{Digest, Sha1};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

/// The RFC 6455 magic GUID (fixed by the spec).
const WS_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

/// The broadcast queue's cap: pending wire events before the oldest drop
/// (a slow client must never back-pressure the app's observer thread,
/// §14.3's bounded-buffer rule).
const BROADCAST_CAP: usize = 4_096;

/// The inspector server: bound listener + the recorder it streams from.
pub struct InspectorServer {
    listener: TcpListener,
    recorder: CausalRecorder,
    app: String,
}

impl InspectorServer {
    /// Binds `addr` (`"127.0.0.1:0"` picks a free port — the demo default).
    ///
    /// # Errors
    ///
    /// Propagates [`std::io::Error`] from the bind.
    pub fn bind(
        addr: &str,
        recorder: CausalRecorder,
        app: impl Into<String>,
    ) -> std::io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        Ok(Self {
            listener,
            recorder,
            app: app.into(),
        })
    }

    /// The bound port (for tests and demos that print the URL).
    #[must_use]
    pub fn port(&self) -> u16 {
        self.listener.local_addr().map_or(0, |addr| addr.port())
    }

    /// The client URL (`http://127.0.0.1:{port}/`).
    #[must_use]
    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}/", self.port())
    }

    /// Accepts and serves connections forever.
    ///
    /// # Panics
    ///
    /// Panics when the accept loop fails (a bound listener failing to accept
    /// is unrecoverable for a devtools server).
    pub fn serve_forever(self) {
        loop {
            let (stream, _peer) = match self.listener.accept() {
                Ok(accepted) => accepted,
                Err(error) => panic!("inspector accept failed: {error}"),
            };
            let recorder = self.recorder.clone();
            let app = self.app.clone();
            std::thread::spawn(move || {
                let _ = serve_connection(stream, &recorder, &app);
            });
        }
    }

    /// Accepts and serves exactly one connection — the test seam.
    ///
    /// # Errors
    ///
    /// Propagates [`std::io::Error`] from the accept or the connection.
    pub fn serve_once(&self) -> std::io::Result<()> {
        let (stream, _peer) = self.listener.accept()?;
        serve_connection(stream, &self.recorder, &self.app)
    }
}

/// Serves one HTTP-or-WS connection to completion.
fn serve_connection(
    stream: TcpStream,
    recorder: &CausalRecorder,
    app: &str,
) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let request = read_request_head(&mut reader)?;
    let websocket = request.upgrade_key.as_deref();

    if let Some(key) = websocket {
        let mut stream = stream;
        accept_websocket(&mut stream, key)?;
        run_session(&mut stream, &mut reader, recorder, app)
    } else {
        serve_html(&stream, &request.path)
    }
}

/// One parsed HTTP request head.
struct RequestHead {
    path: String,
    upgrade_key: Option<String>,
}

/// Reads the request head (through the blank line), case-insensitive header
/// names. Body bytes (none for our two routes) are left in the reader.
fn read_request_head(reader: &mut BufReader<TcpStream>) -> std::io::Result<RequestHead> {
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let path = line.split_whitespace().nth(1).unwrap_or("/").to_string();
    let mut upgrade_key = None;
    loop {
        let mut header = String::new();
        reader.read_line(&mut header)?;
        let trimmed = header.trim_end();
        if trimmed.is_empty() {
            break;
        }
        if let Some((name, value)) = trimmed.split_once(':')
            && name.eq_ignore_ascii_case("sec-websocket-key")
        {
            upgrade_key = Some(value.trim().to_string());
        }
    }
    Ok(RequestHead { path, upgrade_key })
}

/// Serves the embedded HTML client (any non-WS path).
fn serve_html(mut stream: &TcpStream, path: &str) -> std::io::Result<()> {
    let body = if path == "/" || path.starts_with("/index") {
        inspector_html()
    } else {
        format!(
            "<!doctype html><meta charset='utf-8'><title>Alacer inspector</title>\
             <p>Unknown path {path}. The client lives at <a href='/'>/</a>, \
             the protocol at <code>/ws</code>.</p>"
        )
    };
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()
}

/// The RFC 6455 server-side handshake.
fn accept_websocket(stream: &mut TcpStream, key: &str) -> std::io::Result<()> {
    let mut hasher = Sha1::new();
    hasher.update(key.as_bytes());
    hasher.update(WS_GUID.as_bytes());
    let accept = base64::engine::general_purpose::STANDARD.encode(hasher.finalize());
    let response = format!(
        "HTTP/1.1 101 Switching Protocols\r\n\
         Upgrade: websocket\r\n\
         Connection: Upgrade\r\n\
         Sec-WebSocket-Accept: {accept}\r\n\r\n"
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()
}

/// One session: hello, then broadcast drain + client controls, until close.
fn run_session(
    stream: &mut TcpStream,
    reader: &mut BufReader<TcpStream>,
    recorder: &CausalRecorder,
    app: &str,
) -> std::io::Result<()> {
    let hello = Hello {
        kind: "hello",
        version: PROTOCOL_VERSION,
        app: app.to_string(),
    };
    send_text(
        stream,
        &serde_json::to_string(&hello).expect("hello serializes"),
    )?;

    loop {
        // Push everything recorded since the last drain (oldest first).
        let events = recorder.take_broadcast();
        let mut had_work = !events.is_empty();
        for event in events {
            let payload = serde_json::to_string(&event).expect("wire events serialize");
            send_text(stream, &payload)?;
        }
        // Non-blocking read of whatever the client sent (controls, pings).
        match try_read_client_frame(reader)? {
            ClientFrame::None => {}
            ClientFrame::Text(text) => {
                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(ClientMessage::Clear) => recorder.clear(),
                    Ok(ClientMessage::Bye) | Err(_) => break,
                }
                had_work = true;
            }
            ClientFrame::Ping(payload) => {
                send_frame(stream, 0xA, &payload)?;
                had_work = true;
            }
            ClientFrame::Pong => break,
            // RFC 6455 §5.5.1: echo the close reason back, then end.
            ClientFrame::Close(payload) => {
                let _ = send_frame(stream, 0x8, &payload);
                break;
            }
        }
        if !had_work {
            // No events and no client traffic: brief sleep, then re-poll.
            // A condvar would be tighter; a 2 ms poll keeps the session
            // simple and the latency invisible for a devtools stream.
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    }
    let _ = send_frame(stream, 0x8, &[]);
    Ok(())
}

/// What the client sent, one frame at a time.
enum ClientFrame {
    None,
    Text(String),
    Ping(Vec<u8>),
    Pong,
    Close(Vec<u8>),
}

/// Reads one client frame without blocking (poll mode): returns
/// [`ClientFrame::None`] when no complete frame is buffered.
fn try_read_client_frame(reader: &mut BufReader<TcpStream>) -> std::io::Result<ClientFrame> {
    reader.get_mut().set_nonblocking(true)?;
    let result = read_client_frame(reader);
    reader.get_mut().set_nonblocking(false)?;
    result
}

/// Reads one client frame (blocking on the socket, not on our buffer).
fn read_client_frame(reader: &mut BufReader<TcpStream>) -> std::io::Result<ClientFrame> {
    let mut header = [0_u8; 2];
    match reader.read_exact(&mut header) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => {
            return Ok(ClientFrame::Close(Vec::new()));
        }
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
            return Ok(ClientFrame::None);
        }
        Err(error) => return Err(error),
    }
    let opcode = header[0] & 0x0F;
    let masked = header[1] & 0x80 != 0;
    let mut length = u64::from(header[1] & 0x7F);
    if length == 126 {
        let mut extended = [0_u8; 2];
        reader.read_exact(&mut extended)?;
        length = u64::from(u16::from_be_bytes(extended));
    } else if length == 127 {
        let mut extended = [0_u8; 8];
        reader.read_exact(&mut extended)?;
        length = u64::from_be_bytes(extended);
    }
    let mut mask = [0_u8; 4];
    if masked {
        reader.read_exact(&mut mask)?;
    }
    let mut payload = vec![0_u8; usize::try_from(length).unwrap_or(0)];
    if !payload.is_empty() {
        reader.read_exact(&mut payload)?;
    }
    if masked {
        for (index, byte) in payload.iter_mut().enumerate() {
            *byte ^= mask[index % 4];
        }
    }
    Ok(match opcode {
        0x1 | 0x2 => ClientFrame::Text(String::from_utf8_lossy(&payload).into_owned()),
        0x8 => ClientFrame::Close(payload),
        0x9 => ClientFrame::Ping(payload),
        0xA => ClientFrame::Pong,
        _ => ClientFrame::None,
    })
}

/// Sends one server frame (FIN + opcode; servers never mask, RFC 6455 §5.1).
fn send_frame(stream: &mut TcpStream, opcode: u8, payload: &[u8]) -> std::io::Result<()> {
    let mut frame = Vec::with_capacity(payload.len() + 10);
    frame.push(0x80 | opcode);
    let length = payload.len();
    if length < 126 {
        frame.push(alacer_core::num::usize_to_u8(length));
    } else if u16::try_from(length).is_ok() {
        frame.push(126);
        frame.extend_from_slice(&alacer_core::num::usize_to_u16(length).to_be_bytes());
    } else {
        frame.push(127);
        frame.extend_from_slice(&(length as u64).to_be_bytes());
    }
    frame.extend_from_slice(payload);
    stream.write_all(&frame)?;
    stream.flush()
}

/// Sends one text frame.
fn send_text(stream: &mut TcpStream, text: &str) -> std::io::Result<()> {
    send_frame(stream, 0x1, text.as_bytes())
}

/// The embedded client: the standalone inspector app (§13.3 "the inspector
/// is a standalone app").
#[must_use]
pub fn inspector_html() -> String {
    include_str!("inspector.html").to_string()
}

/// The broadcast push the recorder handle performs (kept here so the wire
/// vocabulary and its cap live beside the session that drains them).
pub(crate) fn push_broadcast(queue: &mut std::collections::VecDeque<WireEvent>, event: WireEvent) {
    if queue.len() >= BROADCAST_CAP {
        queue.pop_front();
    }
    queue.push_back(event);
}

/// Makes the recorder's broadcast conversion available to the recorder
/// module without a public cross-module dependency cycle. Total: every
/// `InspectEvent` kind has a wire shape.
pub(crate) fn wire_event_of(event: &alacer_core::inspect::InspectEvent) -> WireEvent {
    match event {
        alacer_core::inspect::InspectEvent::Edge(activation) => wire::wire_edge(activation),
        alacer_core::inspect::InspectEvent::Write(write) => wire::wire_write(write),
        alacer_core::inspect::InspectEvent::Input(input) => WireEvent::Input {
            input: wire::input_name(input.kind).to_string(),
            target: input.target.map(alacer_core::NodeId::raw),
        },
        alacer_core::inspect::InspectEvent::Frame(summary) => wire::wire_frame(summary),
    }
}
