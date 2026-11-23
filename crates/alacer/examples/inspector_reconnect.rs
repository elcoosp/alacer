//! The gate that persists across reconnects (M145): the counter runs
//! with the inspector attached, the clicks stream live — and after the
//! window closes, a scripted *first panel* connects to the same
//! server, gates its live stream to the frames tier with
//! `{"kind":"subscribe","tiers":["frames"]}`, and leaves. A scripted
//! *reconnecting panel* then connects to the same server and reads its
//! greeting: the gate comes back before the panel asks for anything —
//! `"gate":["frames"]` beside the cursor source the `facts` field
//! carries — the server remembering the panel (ADR-0149's shape, the
//! M140 precedent at the gate tier). The facts the muted tiers record
//! stay silent on the reconnect; the frames cross; the replay ask
//! answers whole either way.
//!
//! Run with `cargo run -p alacer --example inspector_reconnect`, click
//! a few times, close the window — the reconnect walkthrough prints
//! what the remembered gate carried in the terminal.
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU
//! machines).

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
                // wrote this signal" — the write fact the remembered
                // gate leaves silent, the frames it streams).
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

    // Port 0: the reconnect walkthrough binds its own port and prints
    // it — the demo canon's 9217 stays free for the other inspector
    // demos.
    let inspector = attach_inspector(
        &mut pipeline,
        InspectorOptions::new()
            .app("the reconnect demo")
            .addr("127.0.0.1:0"),
    )
    .expect("the reconnect demo's inspector binds");
    let port = inspector.port();
    println!(
        "inspector: {} — click a few times, then close the window",
        inspector.url()
    );

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer reconnect demo",
        (360.0, 200.0),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window is closed; the inspector's accept loop still serves —
    // and the demo's own handle can still record the facts the
    // walkthrough streams: the gate's two beats, named.
    let mut handle = inspector.recorder().clone_handle();
    reconnect_walkthrough(port, &mut handle);
}

/// Connects twice to the running inspector: the first panel gates its
/// stream to the frames tier and leaves; the reconnecting panel reads
/// its greeting — the remembered gate beside the cursor source — and
/// prints which facts crossed it without re-asking. Std-lib only: the
/// protocol is the product, no client crate rides along.
fn reconnect_walkthrough(port: u16, handle: &mut impl alacer_core::inspect::InspectObserver) {
    // — Session one: the panel that sets the gate. —
    let Ok(socket) = TcpStream::connect(("127.0.0.1", port)) else {
        eprintln!("reconnect walkthrough: the inspector's port went quiet; nothing to ask");
        return;
    };
    let Ok(mut first) = handshake(socket.try_clone().expect("socket clone")) else {
        eprintln!("reconnect walkthrough: the handshake refused; nothing to gate");
        return;
    };
    let hello = read_text_frame(&mut first).expect("first hello");
    println!("first panel:  {hello}");

    // The ask: the live stream gated to the frames tier.
    send_client_text(&socket, br#"{"kind":"subscribe","tiers":["frames"]}"#);
    std::thread::sleep(Duration::from_millis(80));
    // One fact the gate leaves silent (a write), one it streams (a
    // frame), then the panel leaves — the ask is what the server keeps.
    handle.observe(alacer_core::inspect::InspectEvent::Write(
        alacer_core::inspect::SignalWrite {
            signal: 1,
            sequence: 1,
            origin: std::borrow::Cow::Borrowed("the reconnect demo"),
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
    send_client_text(&socket, br#"{"kind":"bye"}"#);
    std::thread::sleep(Duration::from_millis(200));

    // — Session two: the reconnect. Same server, no ask sent yet. —
    let Ok(socket) = TcpStream::connect(("127.0.0.1", port)) else {
        eprintln!("reconnect walkthrough: the reconnect refused; nothing to read");
        return;
    };
    let Ok(mut second) = handshake(socket.try_clone().expect("socket clone")) else {
        eprintln!("reconnect walkthrough: the reconnect's handshake refused");
        return;
    };
    let hello = read_text_frame(&mut second).expect("reconnect hello");
    println!("reconnect:    {hello}");
    let remembered = hello.contains(r#""gate":["frames"]"#);
    println!(
        "reconnect:    the gate came back {} — the greeting named it, the panel re-asked nothing",
        if remembered { "applied" } else { "NOT applied" }
    );

    // One write (muted) and one frame (asked) recorded after the
    // reconnect: only the frame crosses the remembered gate.
    handle.observe(alacer_core::inspect::InspectEvent::Write(
        alacer_core::inspect::SignalWrite {
            signal: 1,
            sequence: 2,
            origin: std::borrow::Cow::Borrowed("the reconnect demo"),
        },
    ));
    handle.observe(alacer_core::inspect::InspectEvent::Frame(
        alacer_core::inspect::FrameSummary {
            number: 2,
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

    second
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(500)))
        .expect("read timeout");
    let mut crossed = 0_usize;
    let mut frames = 0_usize;
    while let Ok(event) = read_text_frame(&mut second) {
        crossed += 1;
        if event.contains(r#""kind":"frame""#) {
            frames += 1;
        }
        println!("streamed: {event}");
    }
    println!(
        "reconnect:    {crossed} fact(s) crossed the remembered gate, {frames} frame(s) among \
         them — the writes stayed silent, and the replay ask answers whole either way. The \
         gate the first panel set is the gate the server remembers."
    );
}

/// Performs the RFC 6455 handshake with the spec's own sample key and
/// drains the response head.
fn handshake(socket: TcpStream) -> std::io::Result<BufReader<TcpStream>> {
    let mut writer = socket.try_clone()?;
    writer.write_all(
        b"GET /ws HTTP/1.1\r\nHost: localhost\r\n\
Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\r\n",
    )?;
    let mut reader = BufReader::new(socket);
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
    Ok(reader)
}

/// Sends one masked client text frame (the RFC's client-side masking
/// rule) carrying `text` as the payload.
fn send_client_text(mut socket: &TcpStream, text: &[u8]) {
    let len = u8::try_from(text.len()).unwrap_or_default();
    let mut frame = vec![0x81_u8, 0x80 | len];
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
