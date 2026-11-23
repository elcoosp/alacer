//! The region ask: the counter runs with the inspector attached, the
//! clicks stream live — and after the window closes, a scripted client
//! connects to the *same* server and asks the wire's
//! `{"kind":"region_ask","rect":[x,y,w,h]}` control. The answer is one
//! event — the nodes whose bounds the rect touches, in tree order (the
//! paint order's own), the ask echoed beside the answer: a rect naming
//! no positive area touches nothing, an empty list the honest absence.
//! The matches ride the shape the dump ask answers — never the values,
//! the selections, the hints the secret-carrying rule keeps off the
//! wire.
//!
//! Run with `cargo run -p alacer --example inspector_region`, click a
//! few times, close the window — the walkthrough prints the touched
//! nodes in the terminal. `ALACER_SOFTWARE=1` forces the softbuffer
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
                // Every click lands a frame; every frame's a11y pass
                // hands the resolved tree to the inspector through the
                // observer's tree door — the state the region ask
                // answers from.
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

    // Port 0: the region walkthrough binds its own port and prints it —
    // the demo canon's 9217 stays free for the other inspector demos.
    let inspector = attach_inspector(
        &mut pipeline,
        InspectorOptions::new()
            .app("the region ask demo")
            .addr("127.0.0.1:0"),
    )
    .expect("the region ask demo's inspector binds");
    let port = inspector.port();
    println!(
        "inspector: {} — click a few times, then close the window",
        inspector.url()
    );

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer region ask demo",
        (360.0, 200.0),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window is closed; the inspector's accept loop still serves.
    // The late panel asks the touched nodes: the whole window first,
    // a corner region beside it, the inverted rect's honest absence
    // last.
    region_walkthrough(port);
}

/// Connects to the running inspector, sends the `region_ask` controls,
/// and prints the touched nodes — the whole window, a corner region,
/// the inverted rect's honest absence. Std-lib only: the protocol is
/// the product, no client crate rides along.
fn region_walkthrough(port: u16) {
    let Ok(mut socket) = TcpStream::connect(("127.0.0.1", port)) else {
        eprintln!("region ask walkthrough: the inspector's port went quiet; nothing to ask");
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

    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(700)))
        .expect("read timeout");
    // The whole window: every node whose bounds the frame touches —
    // the root column, the label, the button beside it, tree order.
    ask_and_print(
        &socket,
        &mut reader,
        br#"{"kind":"region_ask","rect":[0.0,0.0,360.0,200.0]}"#,
    );
    // A corner region: whatever stands there — the matches the paint
    // order would composite at that corner.
    ask_and_print(
        &socket,
        &mut reader,
        br#"{"kind":"region_ask","rect":[16.0,16.0,120.0,40.0]}"#,
    );
    // The honest absence: inverted extents name no area — the empty
    // list, never a guessed normalization.
    ask_and_print(
        &socket,
        &mut reader,
        br#"{"kind":"region_ask","rect":[300.0,150.0,-40.0,-40.0]}"#,
    );
    println!(
        "region: the matches are the state the observer's door retained — \
         no ring drains, the broadcast queue untouched, the field's \
         contents never in the answer."
    );
}

/// Sends one region ask and prints the answer.
fn ask_and_print(socket: &TcpStream, reader: &mut BufReader<TcpStream>, text: &[u8]) {
    let mut frame = vec![0x81_u8, 0x80 | u8::try_from(text.len()).unwrap_or_default()];
    frame.extend_from_slice(&[0_u8, 0, 0, 0]);
    frame.extend_from_slice(text);
    socket
        .try_clone()
        .expect("socket clone")
        .write_all(&frame)
        .expect("client ask");
    if let Ok(region) = read_text_frame(reader) {
        println!("region: {region}");
    }
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
