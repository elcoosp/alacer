//! The point ask: the counter runs with the inspector attached, the
//! clicks stream live — and after the window closes, a scripted client
//! connects to the *same* server and composes two asks on the wire:
//! the `{"kind":"tree_dump"}` control answers the resolved tree's
//! shape, the walkthrough reads the button's bounds out of it, and the
//! `{"kind":"hit_ask","point":[x,y]}` control answers which node that
//! point touches — one event, the walk's landed-or-refused shape: the
//! ancestry the walk descended through (root-first, the hit node
//! last), the refusal a named species (`no_tree`, `no_geometry`,
//! `outside`, `malformed`). The shape the secret-carrying rule keeps
//! off the wire — a field's value, its selection, the placeholder and
//! description beside them — has no key in either answer.
//!
//! Run with `cargo run -p alacer --example inspector_hit`, click a few
//! times, close the window — the walkthrough prints the landing and the
//! miss in the terminal. `ALACER_SOFTWARE=1` forces the softbuffer path
//! (VMs, CI, no-GPU machines).

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
                // observer's tree door — the state both asks answer
                // from.
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

    // Port 0: the point walkthrough binds its own port and prints it —
    // the demo canon's 9217 stays free for the other inspector demos.
    let inspector = attach_inspector(
        &mut pipeline,
        InspectorOptions::new()
            .app("the point ask demo")
            .addr("127.0.0.1:0"),
    )
    .expect("the point ask demo's inspector binds");
    let port = inspector.port();
    println!(
        "inspector: {} — click a few times, then close the window",
        inspector.url()
    );

    if let Err(error) = alacer::run_window(
        pipeline,
        "Alacer point ask demo",
        (360.0, 200.0),
        backend_choice(),
    ) {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }

    // The window is closed; the inspector's accept loop still serves.
    // The late panel composes the asks: the shape first, the point
    // beside it — the verdict read against the shape it asked for.
    hit_walkthrough(port);
}

/// Connects to the running inspector, composes the dump ask and the
/// point ask, and prints the walk's verdict — the landing's ancestry,
/// the miss's species. Std-lib only: the protocol is the product, no
/// client crate rides along.
fn hit_walkthrough(port: u16) {
    let Ok(mut socket) = TcpStream::connect(("127.0.0.1", port)) else {
        eprintln!("point ask walkthrough: the inspector's port went quiet; nothing to ask");
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

    // The shape ask first: the dump answers the resolved tree, the
    // walkthrough reads the button's bounds out of it — the composition
    // the ask family invites (the point read against the shape it
    // asked for).
    send_ask(&socket, br#"{"kind":"tree_dump"}"#);
    reader
        .get_ref()
        .set_read_timeout(Some(Duration::from_millis(700)))
        .expect("read timeout");
    let tree = match read_text_frame(&mut reader) {
        Ok(tree) => tree,
        Err(error) => {
            eprintln!("point ask walkthrough: the shape ask went unanswered: {error}");
            return;
        }
    };
    let Some(bounds) = bounds_beside_label(&tree, "Increment") else {
        eprintln!("point ask walkthrough: the button's bounds never rode the shape");
        return;
    };
    println!("tree: the Increment button sits at {bounds:?}");

    // The point ask, landed: the button's own center — the walk
    // descends the ancestry the answer carries (root-first, the hit
    // node last).
    let center = [bounds[0] + bounds[2] / 2.0, bounds[1] + bounds[3] / 2.0];
    send_ask(
        &socket,
        format!(
            r#"{{"kind":"hit_ask","point":[{},{}]}}"#,
            center[0], center[1]
        )
        .as_bytes(),
    );
    if let Ok(hit) = read_text_frame(&mut reader) {
        println!("hit: {hit}");
    }

    // The point ask, missed: a point outside every node's geometry —
    // the refusal a named species, never a collapsed answer.
    send_ask(&socket, br#"{"kind":"hit_ask","point":[1000.0,1000.0]}"#);
    if let Ok(hit) = read_text_frame(&mut reader) {
        println!("hit: {hit}");
    }
    println!(
        "hit: the verdict is the state the observer's door retained — \
         no ring drains, the broadcast queue untouched, the field's \
         contents never in the answer."
    );
}

/// Reads the `[x, y, w, h]` bounds quadruple that rides immediately
/// beside `label`'s own entry in the tree event's JSON — the shape the
/// dump answer speaks, read with the std library only (`serde_json`
/// prints the node fields in declaration order, the label beside the
/// bounds it was laid out with).
fn bounds_beside_label(tree: &str, label: &str) -> Option<[f32; 4]> {
    let needle = format!(r#""label":"{label}""#);
    let after = tree.split(&needle).nth(1)?;
    let bounds_at = after.find(r#""bounds":["#)?;
    let digits = &after[bounds_at + r#""bounds":["#.len()..];
    let end = digits.find(']')?;
    let mut quad = [0.0_f32; 4];
    for (slot, word) in quad.iter_mut().zip(digits[..end].split(',')) {
        *slot = word.trim().parse().ok()?;
    }
    Some(quad)
}

/// Sends one client text frame (masked, RFC 6455 §5.3 — clients mask).
fn send_ask(socket: &TcpStream, text: &[u8]) {
    let mut frame = vec![0x81_u8, 0x80 | u8::try_from(text.len()).unwrap_or_default()];
    frame.extend_from_slice(&[0_u8, 0, 0, 0]);
    frame.extend_from_slice(text);
    socket
        .try_clone()
        .expect("socket clone")
        .write_all(&frame)
        .expect("client ask");
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
