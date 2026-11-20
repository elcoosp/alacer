//! The late client's replay (M139): the counter runs with the inspector
//! attached, the clicks stream live — and after the window closes, a
//! scripted client connects to the *same* server and asks the wire's
//! one control the broadcast queue cannot answer: `{"kind":"replay"}`.
//! The burst it reads back is the rings' durable history — the facts a
//! panel that connects after a first panel's drain never sees through
//! the shared queue alone.
//!
//! Run with `cargo run -p alacer --example inspector_replay`, click a
//! few times, close the window — the replay walkthrough prints the
//! burst in the terminal. `ALACER_SOFTWARE=1` forces the softbuffer
//! path (VMs, CI, no-GPU machines).

use alacer::inspect::{InspectorOptions, attach_inspector};
use alacer::pipeline_default;
use alacer::platform::BackendPreference;
use alacer::widgets::{BoxedWidget as _, Button, Column, Label};
use alacer_core::flux::RenderBinding;
use reactive_graph::traits::{GetUntracked as _, Set as _};
use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::net::TcpStream;
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
                // wrote this signal" — §13.3's question the burst answers
                // for the client that was never connected).
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

    // Port 0: the replay walkthrough binds its own port and prints it —
    // the demo canon's 9217 stays free for the other inspector demos.
    let inspector = attach_inspector(
        &mut pipeline,
        InspectorOptions::new()
            .app("the replay demo")
            .addr("127.0.0.1:0"),
    )
    .expect("the replay demo's inspector binds");
    let port = inspector.port();
    println!(
        "inspector: {} — click a few times, then close the window",
        inspector.url()
    );

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer replay demo",
        (360.0, 200.0),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window is closed; the inspector's accept loop still serves.
    // The late client: the panel a second operator opens after the
    // session lived — its ask reads the rings the broadcast queue never
    // held for it.
    replay_walkthrough(port);
}

/// Connects to the running inspector, sends the `replay` control, and
/// prints the burst — the rings' durable history, oldest first per
/// trail. Std-lib only: the protocol is the product, no client crate
/// rides along.
fn replay_walkthrough(port: u16) {
    let Ok(mut socket) = TcpStream::connect(("127.0.0.1", port)) else {
        eprintln!("replay walkthrough: the inspector's port went quiet; nothing to ask");
        return;
    };
    // The RFC 6455 handshake with the spec's own sample key.
    socket
        .write_all(
            b"GET /ws HTTP/1.1\r\nHost: localhost\r\n\
Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\r\n",
        )
        .expect("handshake request");
    let mut reader = BufReader::new(socket.try_clone().expect("socket clone"));
    let mut status = String::new();
    reader.read_line(&mut status).expect("status line");
    assert!(status.starts_with("HTTP/1.1 101"), "upgraded: {status}");
    loop {
        let mut header = String::new();
        reader.read_line(&mut header).expect("response header");
        if header.trim_end().is_empty() {
            break;
        }
    }
    let hello = read_text_frame(&mut reader).expect("hello");
    println!("server: {hello}");

    // The ask.
    let text = br#"{"kind":"replay"}"#;
    let mut frame = vec![0x81_u8, 0x80 | u8::try_from(text.len()).unwrap_or_default()];
    frame.extend_from_slice(&[0_u8, 0, 0, 0]);
    frame.extend_from_slice(text);
    reader.get_ref().write_all(&frame).expect("replay ask");

    // The burst: read whatever the rings answer within the window.
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(700)))
        .expect("read timeout");
    let mut seen = 0_usize;
    while let Ok(payload) = read_text_frame(&mut reader) {
        seen += 1;
        if seen <= 12 {
            println!("replay: {payload}");
        }
    }
    println!("replay: {seen} facts the rings held — the late client caught up.");
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
