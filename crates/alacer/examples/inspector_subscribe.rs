//! The subscribe ask (M142): the counter runs with the inspector
//! attached, the clicks stream live — and after the window closes, a
//! scripted client connects to the *same* server, gates its live
//! stream to the frames tier with `{"kind":"subscribe","tiers":
//! ["frames"]}`, and watches the facts the demo records afterward:
//! the frames cross, the writes and edges stay silent — the panel's
//! own silence, never a loss (the rings keep recording everything,
//! and the replay ask answers whole either way).
//!
//! Run with `cargo run -p alacer --example inspector_subscribe`, click
//! a few times, close the window — the subscribe walkthrough prints
//! what the gated stream carried in the terminal. `ALACER_SOFTWARE=1`
//! forces the softbuffer path (VMs, CI, no-GPU machines).

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
                // wrote this signal" — the edge and write facts the
                // gated panel below leaves silent, the frames it
                // reads).
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

    // Port 0: the subscribe walkthrough binds its own port and prints
    // it — the demo canon's 9217 stays free for the other inspector
    // demos.
    let inspector = attach_inspector(
        &mut pipeline,
        InspectorOptions::new()
            .app("the subscribe demo")
            .addr("127.0.0.1:0"),
    )
    .expect("the subscribe demo's inspector binds");
    let port = inspector.port();
    println!(
        "inspector: {} — click a few times, then close the window",
        inspector.url()
    );

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer subscribe demo",
        (360.0, 200.0),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window is closed; the inspector's accept loop still serves —
    // and the demo's own handle can still record the facts the
    // walkthrough streams: the gated panel's live tier, named.
    let mut handle = inspector.recorder().clone_handle();
    subscribe_walkthrough(port, &mut handle);
}

/// Connects to the running inspector, sends the `subscribe` ask (the
/// live stream gated to the frames tier), records one edge, one write,
/// and one frame through the demo's own handle, and prints which of
/// the three crossed. Std-lib only: the protocol is the product, no
/// client crate rides along.
fn subscribe_walkthrough(port: u16, handle: &mut impl alacer_core::inspect::InspectObserver) {
    let Ok(mut socket) = TcpStream::connect(("127.0.0.1", port)) else {
        eprintln!("subscribe walkthrough: the inspector's port went quiet; nothing to ask");
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

    // The ask: the live stream gated to the frames tier.
    let text = br#"{"kind":"subscribe","tiers":["frames"]}"#;
    let mut frame = vec![0x81_u8, 0x80 | u8::try_from(text.len()).unwrap_or_default()];
    frame.extend_from_slice(&[0_u8, 0, 0, 0]);
    frame.extend_from_slice(text);
    reader.get_ref().write_all(&frame).expect("subscribe ask");
    std::thread::sleep(Duration::from_millis(80));

    // Three facts the demo records itself — an edge, a write, a frame:
    // only the frame's tier was asked for, only the frame crosses.
    handle.observe(alacer_core::inspect::InspectEvent::Edge(
        alacer_core::inspect::EdgeActivation {
            binding: 1,
            signal: 1,
            node: alacer_core::arena::NodeId::allocate(),
            masks: alacer_core::Change::PAINT,
        },
    ));
    handle.observe(alacer_core::inspect::InspectEvent::Write(
        alacer_core::inspect::SignalWrite {
            signal: 1,
            sequence: 1,
            origin: std::borrow::Cow::Borrowed("the subscribe demo"),
        },
    ));
    handle.observe(alacer_core::inspect::InspectEvent::Frame(
        alacer_core::inspect::FrameSummary {
            number: 1,
            layout_ran: true,
            paint_ran: true,
            a11y_ran: false,
            focus_changed: false,
            degradation_step: alacer_core::passes::DegradationStep::None,
            damage: alacer_core::Damage::NONE,
            viewport: alacer_core::geom::Size::new(360.0, 200.0),
            a11y_updated: 0,
            a11y_removed: 0,
            total_nanos: 0,
            passes: Vec::new(),
            dirty: Vec::new(),
            dirty_truncated: 0,
        },
    ));
    std::thread::sleep(Duration::from_millis(200));

    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(500)))
        .expect("read timeout");
    let mut crossed = 0_usize;
    while let Ok(event) = read_text_frame(&mut reader) {
        crossed += 1;
        println!("streamed: {event}");
    }
    println!(
        "subscribe: {crossed} fact(s) crossed the gated stream — the frame's \
         tier only. The rings recorded all three: the replay ask answers \
         whole, the gate the panel's own silence, never a loss."
    );
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
