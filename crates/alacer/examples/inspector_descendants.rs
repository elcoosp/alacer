//! The descendant ask: the counter runs with the inspector attached, the
//! clicks stream live — and after the window closes, a scripted client
//! connects to the *same* server and composes the ask family two doors
//! deep: a `find_ask` names the button, the node id the answer carries
//! feeds a `descendants_ask`, and the answer is one event — the subtree
//! that stands beneath, tree order, the asked node the anchor the ask
//! already names (never its own answer's first row). An id naming
//! nothing standing answers the `no_node` miss, a tree that has not
//! resolved yet the `no_tree` one — never a collapsed `false`. A leaf's
//! landing carries the honest absence: nothing beneath, never a
//! refusal. The subtree rides the shape the dump ask answers — never
//! the values, the selections, the hints the secret-carrying rule
//! keeps off the wire.
//!
//! Run with `cargo run -p alacer --example inspector_descendants`,
//! click a few times, close the window — the walkthrough prints the
//! subtrees in the terminal. `ALACER_SOFTWARE=1` forces the softbuffer
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
                // observer's tree door — the state the descendant ask
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

    // Port 0: the descendant walkthrough binds its own port and prints
    // it — the demo canon's 9217 stays free for the other inspector
    // demos.
    let inspector = attach_inspector(
        &mut pipeline,
        InspectorOptions::new()
            .app("the descendant ask demo")
            .addr("127.0.0.1:0"),
    )
    .expect("the descendant ask demo's inspector binds");
    let port = inspector.port();
    println!(
        "inspector: {} — click a few times, then close the window",
        inspector.url()
    );

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer descendant ask demo",
        (360.0, 200.0),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window is closed; the inspector's accept loop still serves.
    // The late panel composes the asks: the word ask names the button,
    // the id the answer carries feeds the descendant ask, the leaf's
    // honest absence and the unknown id's miss ride beside them.
    descendants_walkthrough(port);
}

/// Connects to the running inspector and composes the asks: the word
/// ask first (the node id its answer carries feeding the walk), the
/// descendant ask beside it, the leaf's absence and the unknown id's
/// miss last. Std-lib only: the protocol is the product, no client
/// crate rides along.
fn descendants_walkthrough(port: u16) {
    let Ok(mut socket) = TcpStream::connect(("127.0.0.1", port)) else {
        eprintln!("descendant walkthrough: the inspector's port went quiet; nothing to ask");
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
    // The word ask names the button: the narrowed needle's first match
    // carries the node id the descendant ask rides — the ask family
    // composing.
    let find = ask(
        &socket,
        &mut reader,
        br#"{"kind":"find_ask","query":"increment","role":"button"}"#,
    )
    .unwrap_or_default();
    println!("find: {find}");
    if let Some(node) = first_match_node_id(&find) {
        let ask_text = format!(r#"{{"kind":"descendants_ask","node":{node}}}"#);
        ask_and_print(&socket, &mut reader, ask_text.as_bytes());
    } else {
        println!("find: the honest absence — no name carried the needle, no subtree to walk");
    }
    // The leaf's honest absence: the counter's label stands under
    // nothing — the landing names the absence, never a refusal.
    if let Some(label) = first_match_node_id(
        &ask(
            &socket,
            &mut reader,
            br#"{"kind":"find_ask","query":"clicked","role":"label"}"#,
        )
        .unwrap_or_default(),
    ) {
        let ask_text = format!(r#"{{"kind":"descendants_ask","node":{label}}}"#);
        ask_and_print(&socket, &mut reader, ask_text.as_bytes());
    }
    // The named miss: an id the tree does not carry answers its
    // species, never a collapsed false.
    ask_and_print(
        &socket,
        &mut reader,
        br#"{"kind":"descendants_ask","node":987654321}"#,
    );
    println!(
        "descendants: the subtree is the state the observer's door retained — \
         no ring drains, the broadcast queue untouched, the field's \
         contents never in the answer."
    );
}

/// Scans the word ask's answer for its first match's node id — the
/// composition's glue, no JSON parser riding along.
fn first_match_node_id(find: &str) -> Option<u64> {
    let marker = r#""matches":[{"node":"#;
    let start = find.find(marker)? + marker.len();
    let rest = &find[start..];
    let end = rest.find(',').unwrap_or(rest.len());
    rest[..end].parse().ok()
}

/// Sends one ask and returns the answer's text.
fn ask(socket: &TcpStream, reader: &mut BufReader<TcpStream>, text: &[u8]) -> Option<String> {
    let mut frame = vec![0x81_u8, 0x80 | u8::try_from(text.len()).unwrap_or_default()];
    frame.extend_from_slice(&[0_u8, 0, 0, 0]);
    frame.extend_from_slice(text);
    socket
        .try_clone()
        .expect("socket clone")
        .write_all(&frame)
        .expect("client ask");
    read_text_frame(reader).ok()
}

/// Sends one ask and prints the answer.
fn ask_and_print(socket: &TcpStream, reader: &mut BufReader<TcpStream>, text: &[u8]) {
    if let Some(answer) = ask(socket, reader, text) {
        println!("descendants: {answer}");
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
