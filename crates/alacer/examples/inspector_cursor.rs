//! The replay cursor (M140): the counter runs with the inspector
//! attached, the clicks stream live — and the walkthrough shows what the
//! wire's optional `after` cursor buys the second panel. Panel A connects
//! a few seconds in (its greeting names `facts: F_A`), streams the live
//! facts, and stays connected until the window closes. Panel B then
//! connects to the *same* server and asks `{"kind":"replay","after":F_A}`
//! — the burst answers only the facts recorded *after* A's greeting: the
//! history A already carried (the mount's startup facts, any clicks
//! before A connected) stays silent, and the delta rides whole.
//!
//! Run with `cargo run -p alacer --example inspector_cursor`, click a few
//! times (the clicks in the first seconds ride *below* panel A's cursor —
//! the skip the demo shows), then close the window. `ALACER_SOFTWARE=1`
//! forces the softbuffer path (VMs, CI, no-GPU machines).

use alacer::inspect::{InspectorOptions, attach_inspector};
use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, Label};
use alacer_core::flux::RenderBinding;
use reactive_graph::traits::{GetUntracked as _, Set as _};
use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

fn main() {
    let mut pipeline = pipeline_default();
    let root = pipeline.mount_root(Column::new().gap(12.0).padding(16.0).boxed());

    let count = pipeline.flux().signal(0_i32);
    let binding = RenderBinding::new("Clicked 0 times".to_string());
    let label = pipeline.add_child(root, Label::bound(binding.clone()).boxed());
    let _ = pipeline.add_child(
        root,
        Button::new("Increment")
            .on_click(move || {
                // The provenance-tagged write the inspector logs ("who
                // wrote this signal" — §13.3's question the cursor
                // narrows for the panel that asks with its greeting's
                // own fact count).
                alacer_core::flux::Flux::batch_of(|| {
                    let current = count.try_get_untracked().unwrap_or_default();
                    count.set(current + 1);
                });
            })
            .boxed(),
    );

    pipeline.flux_mut().bind_value(
        count,
        label,
        &binding,
        alacer_core::Change::LAYOUT.union(alacer_core::Change::PAINT),
        |n: &i32| format!("Clicked {n} times"),
    );

    // Port 0: the cursor walkthrough binds its own port and prints it —
    // the demo canon's 9217 stays free for the other inspector demos.
    let inspector = attach_inspector(
        &mut pipeline,
        InspectorOptions::new()
            .app("the cursor demo")
            .addr("127.0.0.1:0"),
    )
    .expect("the cursor demo's inspector binds");
    let port = inspector.port();
    println!(
        "inspector: {} — click a few times, then close the window",
        inspector.url()
    );

    // Panel A connects a few seconds in — the greeting it reads names
    // the fact count its cursor rides. Everything recorded before the
    // greeting (the mount's startup facts, the clicks made in the
    // window's first seconds) sits at-or-below the cursor; everything
    // after streams live to A.
    let cursor = Arc::new(AtomicU64::new(0));
    let live = Arc::new(AtomicU64::new(0));
    let closed = Arc::new(AtomicBool::new(false));
    let panel_cursor = Arc::clone(&cursor);
    let panel_live = Arc::clone(&live);
    let panel_closed = Arc::clone(&closed);
    std::thread::spawn(move || panel_a_session(port, &panel_cursor, &panel_live, &panel_closed));

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer cursor demo",
        (360.0, 200.0),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window is closed; the inspector's accept loop still serves.
    // Panel A's thread sees the close, notes its greeting's fact count
    // and the live facts it streamed, and hangs up. Panel B asks with
    // A's cursor.
    closed.store(true, Ordering::Relaxed);
    std::thread::sleep(Duration::from_millis(300));
    let panel_a_facts = cursor.load(Ordering::Relaxed);
    let panel_a_live = live.load(Ordering::Relaxed);
    cursor_walkthrough(port, panel_a_facts, panel_a_live);
}

/// Panel A: connects after the delay, reads the greeting (the cursor's
/// source), then streams the live facts until the window closes — the
/// first panel whose drain the second panel's cursor skips.
fn panel_a_session(
    port: u16,
    cursor: &Arc<AtomicU64>,
    live: &Arc<AtomicU64>,
    closed: &Arc<AtomicBool>,
) {
    std::thread::sleep(Duration::from_secs(3));
    let Ok(socket) = TcpStream::connect(("127.0.0.1", port)) else {
        return;
    };
    if handshake(&socket).is_err() {
        return;
    }
    let mut reader = BufReader::new(socket.try_clone().expect("socket clone"));
    socket
        .set_read_timeout(Some(Duration::from_millis(250)))
        .expect("read timeout");
    // The greeting, exactly once, first — its `facts` field is the
    // number panel B's replay ask carries as its `after` cursor.
    if let Ok(hello) = read_text_frame(&mut reader) {
        let facts = hello
            .split(r#""facts":"#)
            .nth(1)
            .and_then(|tail| tail.split('}').next())
            .and_then(|digits| digits.parse::<u64>().ok())
            .unwrap_or_default();
        cursor.store(facts, Ordering::Relaxed);
    }
    // The live stream, until the window closes (the polling loop reads
    // what arrives; the close flag ends the session).
    while !closed.load(Ordering::Relaxed) {
        if read_text_frame(&mut reader).is_ok() {
            live.fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// Panel B: connects to the same server and asks the replay with panel
/// A's cursor — the burst answers the facts recorded after A's greeting,
/// the history at-or-below it staying silent.
fn cursor_walkthrough(port: u16, panel_a_facts: u64, panel_a_live: u64) {
    let Ok(socket) = TcpStream::connect(("127.0.0.1", port)) else {
        eprintln!("cursor walkthrough: the inspector's port went quiet; nothing to ask");
        return;
    };
    handshake(&socket).expect("panel B's handshake");
    let mut reader = BufReader::new(socket.try_clone().expect("socket clone"));
    let hello = read_text_frame(&mut reader).expect("panel B's hello");
    println!("panel B: {hello}");
    println!(
        "panel A's greeting named {panel_a_facts} facts and streamed \
         {panel_a_live} live — the cursor asks above the greeting"
    );

    // The ask: the cursor rides the field the greeting named.
    let text = format!(r#"{{"kind":"replay","after":{panel_a_facts}}}"#);
    send_client_text(&socket, text.as_bytes());

    // The delta: read whatever the rings answer within the window.
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(700)))
        .expect("read timeout");
    let mut delta = 0_usize;
    while let Ok(payload) = read_text_frame(&mut reader) {
        delta += 1;
        if delta <= 12 {
            println!("delta: {payload}");
        }
    }
    println!(
        "cursor: {delta} facts recorded after panel A's greeting — the history \
         at or below the cursor stayed silent (the facts A streamed live and \
         the burst repeats fold away client-side)."
    );
}

/// The RFC 6455 handshake with the spec's own sample key.
fn handshake(mut socket: &TcpStream) -> std::io::Result<()> {
    socket.write_all(
        b"GET /ws HTTP/1.1\r\nHost: localhost\r\n\
Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\r\n",
    )?;
    let mut reader = BufReader::new(socket.try_clone().expect("socket clone"));
    let mut status = String::new();
    reader.read_line(&mut status)?;
    assert!(status.starts_with("HTTP/1.1 101"), "upgraded: {status}");
    loop {
        let mut header = String::new();
        reader.read_line(&mut header)?;
        if header.trim_end().is_empty() {
            break;
        }
    }
    Ok(())
}

/// Sends one masked client text frame (the RFC's client-side masking
/// rule) carrying `text` as the payload.
fn send_client_text(mut socket: &TcpStream, text: &[u8]) {
    let mut frame = vec![0x81_u8, 0x80 | u8::try_from(text.len()).unwrap_or_default()];
    frame.extend_from_slice(&[0_u8, 0, 0, 0]);
    frame.extend_from_slice(text);
    socket.write_all(&frame).expect("client frame");
}

/// Reads one server text frame (unmasked, RFC 6455) from the socket.
fn read_text_frame(reader: &mut BufReader<TcpStream>) -> std::io::Result<String> {
    let mut header = [0_u8; 2];
    reader.read_exact(&mut header)?;
    let length = usize::try_from(u64::from(header[1] & 0x7F)).unwrap_or_default();
    let mut payload = vec![0_u8; length];
    reader.read_exact(&mut payload)?;
    String::from_utf8(payload)
        .map_err(|error| std::io::Error::other(format!("utf-8 payload: {error}")))
}

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}
