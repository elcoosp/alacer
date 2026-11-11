//! The inspector protocol's integration tests: a real server on a real
//! socket, a raw TCP client doing the RFC 6455 handshake, and the JSON
//! stream pinned against `docs/inspector-protocol.md`.

use alacer_core::inspect::{InputKind, InputRecord, InspectEvent, InspectObserver as _};
use alacer_inspect::{CausalRecorder, InspectorServer};
use std::io::{BufRead as _, BufReader, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

const HANDSHAKE: &[u8] = b"GET /ws HTTP/1.1\r\nHost: localhost\r\n\
Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\r\n";

/// Reads one text frame from the buffered socket.
fn read_text_frame(reader: &mut BufReader<TcpStream>) -> String {
    let mut header = [0_u8; 2];
    reader.read_exact(&mut header).expect("frame header");
    assert_eq!(header[0] & 0x0F, 0x1, "text frame opcode");
    let short = u64::from(header[1] & 0x7F);
    let length = match short {
        126 => {
            let mut extended = [0_u8; 2];
            reader.read_exact(&mut extended).expect("16-bit length");
            u64::from(u16::from_be_bytes(extended))
        }
        127 => panic!("64-bit lengths do not occur in this suite"),
        length => length,
    };
    let mut payload = vec![0_u8; usize::try_from(length).expect("sane length")];
    reader.read_exact(&mut payload).expect("frame payload");
    String::from_utf8(payload).expect("utf-8 payload")
}

/// Connects to the server's `/ws` and returns the buffered stream.
fn connect_ws(port: u16) -> BufReader<TcpStream> {
    let mut socket = TcpStream::connect(("127.0.0.1", port)).expect("connect");
    socket.write_all(HANDSHAKE).expect("handshake request");
    let mut reader = BufReader::new(socket.try_clone().expect("clone"));
    let mut status = String::new();
    reader.read_line(&mut status).expect("status line");
    assert_eq!(
        status.trim_end(),
        "HTTP/1.1 101 Switching Protocols",
        "the RFC's status line"
    );
    // Drain the remaining response headers through the blank line.
    loop {
        let mut header = String::new();
        reader.read_line(&mut header).expect("header line");
        if header.trim_end().is_empty() {
            break;
        }
        let lower = header.to_ascii_lowercase();
        if lower.starts_with("upgrade:") {
            assert!(lower.contains("websocket"), "{header}");
        }
    }
    reader
}

#[test]
fn the_handshake_computes_the_rfc_6455_reference_accept() {
    // The spec's §1.3 test vector: our SHA-1+base64 composition is pinned
    // against the RFC, not against itself.
    use base64::Engine as _;
    use sha1::{Digest, Sha1};
    let mut hasher = Sha1::new();
    hasher.update(b"dGhlIHNhbXBsZSBub25jZQ==258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    let accept = base64::engine::general_purpose::STANDARD.encode(hasher.finalize());
    assert_eq!(accept, "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");
}

#[test]
fn a_session_streams_hello_then_recorded_events_in_order() {
    let recorder = CausalRecorder::new("protocol");
    let server = InspectorServer::bind("127.0.0.1:0", recorder.clone(), "protocol").expect("bind");
    let port = server.port();
    std::thread::spawn(move || server.serve_once());
    let mut reader = connect_ws(port);

    // The greeting names the app and the protocol version.
    let hello = read_text_frame(&mut reader);
    assert_eq!(
        hello, r#"{"kind":"hello","version":1,"app":"protocol"}"#,
        "the documented greeting"
    );

    // Events recorded after the connect land on the socket, in order.
    std::thread::sleep(Duration::from_millis(30));
    recorder
        .clone_handle()
        .observe(InspectEvent::Input(InputRecord {
            kind: InputKind::Wheel,
            target: None,
        }));
    let mut summary = dummy_frame(3);
    summary.paint_ran = true;
    summary.damage = alacer_core::Damage::NONE;
    recorder
        .clone_handle()
        .observe(InspectEvent::Frame(summary));

    let input = read_text_frame(&mut reader);
    assert!(
        input.contains(r#""kind":"input""#) && input.contains(r#""input":"wheel""#),
        "input event: {input}"
    );
    let frame = read_text_frame(&mut reader);
    assert!(
        frame.contains(r#""kind":"frame""#) && frame.contains(r#""number":3"#),
        "frame event: {frame}"
    );

    // Bye ends the session cleanly (the server thread returns).
    let mut socket = reader.get_ref().try_clone().expect("clone");
    // Send a masked client text frame: {"kind":"bye"}.
    let text = br#"{"kind":"bye"}"#;
    let mut frame_bytes = vec![0x81_u8, 0x80 | alacer_core::num::usize_to_u8(text.len())];
    frame_bytes.extend_from_slice(&[0_u8, 0, 0, 0]);
    frame_bytes.extend_from_slice(text);
    socket.write_all(&frame_bytes).expect("bye");
    std::thread::sleep(Duration::from_millis(50));
}

#[test]
fn the_clear_control_resets_the_recording() {
    let recorder = CausalRecorder::new("clear");
    let server = InspectorServer::bind("127.0.0.1:0", recorder.clone(), "clear").expect("bind");
    let port = server.port();
    std::thread::spawn(move || server.serve_once());
    let mut reader = connect_ws(port);
    let _hello = read_text_frame(&mut reader);

    std::thread::sleep(Duration::from_millis(30));
    recorder
        .clone_handle()
        .observe(InspectEvent::Input(InputRecord {
            kind: InputKind::Key,
            target: None,
        }));
    let _input = read_text_frame(&mut reader);
    assert_eq!(recorder.inputs().len(), 1, "recorded before the clear");

    // {"kind":"clear"} — masked client frame.
    let mut socket = reader.get_ref().try_clone().expect("clone");
    let text = br#"{"kind":"clear"}"#;
    let mut frame_bytes = vec![0x81_u8, 0x80 | alacer_core::num::usize_to_u8(text.len())];
    frame_bytes.extend_from_slice(&[0_u8, 0, 0, 0]);
    frame_bytes.extend_from_slice(text);
    socket.write_all(&frame_bytes).expect("clear");
    std::thread::sleep(Duration::from_millis(80));
    assert_eq!(
        recorder.inputs().len(),
        0,
        "the server's clear control reset the recording"
    );
}

#[test]
fn get_root_serves_the_embedded_client() {
    let recorder = CausalRecorder::new("html");
    let server = InspectorServer::bind("127.0.0.1:0", recorder, "html").expect("bind");
    let port = server.port();
    std::thread::spawn(move || server.serve_once());
    let mut socket = TcpStream::connect(("127.0.0.1", port)).expect("connect");
    socket
        .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .expect("request");
    let mut response = String::new();
    BufReader::new(socket)
        .read_to_string(&mut response)
        .expect("response");
    assert!(response.starts_with("HTTP/1.1 200 OK"));
    assert!(response.contains("text/html"));
    assert!(
        response.contains("new WebSocket("),
        "the client speaks the protocol"
    );
    assert!(
        response.contains("Causality Inspector"),
        "the client is the inspector app"
    );
}

#[test]
fn frames_carry_the_full_documented_shape() {
    // The wire conversion is pinned here (the client codes against it).
    use alacer_core::arena::NodeId;
    use alacer_core::inspect::{DirtyNode, EdgeActivation, PassTiming};
    use alacer_core::passes::PassKind;
    use alacer_inspect::wire::{wire_edge, wire_frame};

    let mut summary = dummy_frame(9);
    summary.layout_ran = true;
    summary.paint_ran = true;
    summary.focus_changed = true;
    summary.a11y_updated = 2;
    summary.total_nanos = 12_345;
    summary.passes = vec![
        PassTiming {
            kind: PassKind::Layout,
            ran: true,
            nanos: 1_000,
        },
        PassTiming {
            kind: PassKind::Paint,
            ran: true,
            nanos: 2_000,
        },
    ];
    summary.dirty = vec![DirtyNode {
        node: NodeId::allocate(),
        mask: alacer_core::Change::PAINT,
        role: alacer_core::Role::Button,
        label: "Save".to_string(),
        bounds: Some(alacer_core::geom::Rect::new(
            alacer_core::geom::Point::new(1.0, 2.0),
            alacer_core::geom::Size::new(30.0, 20.0),
        )),
    }];
    let frame = wire_frame(&summary);
    let json = serde_json::to_string(&frame).expect("frame json");
    for fragment in [
        r#""kind":"frame""#,
        r#""number":9"#,
        r#""damage":"none""#,
        r#""name":"layout""#,
        r#""role":"button""#,
        r#""label":"Save""#,
        r#""masks":"paint""#,
        r#""focus_changed":true"#,
    ] {
        assert!(json.contains(fragment), "missing {fragment} in {json}");
    }

    let edge = wire_edge(&EdgeActivation {
        binding: 4,
        signal: 9_001,
        node: NodeId::allocate(),
        masks: alacer_core::Change::LAYOUT,
    });
    let json = serde_json::to_string(&edge).expect("edge json");
    assert!(json.contains(r#""kind":"edge""#));
    assert!(json.contains(r#""binding":4"#));
    assert!(json.contains(r#""signal":9001"#));
    assert!(json.contains(r#""masks":"layout""#));
}

#[test]
fn focus_events_carry_the_documented_flat_shape() {
    // The wire conversion is pinned here (the client codes against it):
    // a landing's endpoints and wrap, a refusal's species, node, and
    // direction — the documented `focus` shape (ADR-0034).
    use alacer_core::arena::NodeId;
    use alacer_core::inspect::FocusRecord;
    use alacer_core::passes::{FocusLanding, FocusMove, FocusMoveOutcome, FocusRefusal};
    use alacer_inspect::wire::wire_focus;

    let node = NodeId::allocate();
    let landing = wire_focus(&FocusRecord {
        movement: FocusMove::Next,
        scope: None,
        rider: alacer_core::inspect::FocusRider::Consumer,
        outcome: FocusMoveOutcome::Moved(FocusLanding {
            from: None,
            to: node,
            wrapped: true,
        }),
    });
    let json = serde_json::to_string(&landing).expect("landing json");
    for fragment in [
        r#""kind":"focus""#,
        r#""movement":"next""#,
        r#""scope":null"#,
        r#""moved":true"#,
        r#""from":null"#,
        r#""wrapped":true"#,
        r#""refusal":null"#,
    ] {
        assert!(json.contains(fragment), "missing {fragment} in {json}");
    }

    let refused = wire_focus(&FocusRecord {
        movement: FocusMove::Right,
        scope: Some(node),
        rider: alacer_core::inspect::FocusRider::Consumer,
        outcome: FocusMoveOutcome::Refused(FocusRefusal::NothingInDirection {
            node,
            direction: FocusMove::Right,
        }),
    });
    let json = serde_json::to_string(&refused).expect("refusal json");
    for fragment in [
        r#""kind":"focus""#,
        r#""movement":"right""#,
        r#""moved":false"#,
        r#""refusal":"nothing_in_direction""#,
        r#""direction":"right""#,
    ] {
        assert!(json.contains(fragment), "missing {fragment} in {json}");
    }
}

#[test]
fn a_session_streams_focus_records_over_the_real_socket() {
    use alacer_core::inspect::FocusRecord;
    use alacer_core::passes::{FocusLanding, FocusMove, FocusMoveOutcome};

    // The recorder serializes the walk's answers as it records them: a
    // focus record lands on the wire in record order, beside the input
    // trail (the protocol's additive rule — new event kind, version 1).
    let recorder = CausalRecorder::new("focus-wire");
    let server =
        InspectorServer::bind("127.0.0.1:0", recorder.clone(), "focus-wire").expect("bind");
    let port = server.port();
    std::thread::spawn(move || server.serve_once());
    let mut reader = connect_ws(port);
    let _hello = read_text_frame(&mut reader);

    std::thread::sleep(Duration::from_millis(30));
    let node = alacer_core::arena::NodeId::allocate();
    recorder
        .clone_handle()
        .observe(InspectEvent::Focus(FocusRecord {
            movement: FocusMove::Down,
            scope: None,
            rider: alacer_core::inspect::FocusRider::Consumer,
            outcome: FocusMoveOutcome::Moved(FocusLanding {
                from: None,
                to: node,
                wrapped: false,
            }),
        }));

    let focus = read_text_frame(&mut reader);
    assert!(
        focus.contains(r#""kind":"focus""#)
            && focus.contains(r#""movement":"down""#)
            && focus.contains(r#""moved":true"#),
        "focus event over the socket: {focus}"
    );
}

/// A minimal frame summary with a real ledger shape.
fn dummy_frame(number: u64) -> alacer_core::inspect::FrameSummary {
    use alacer_core::passes::PassKind;
    alacer_core::inspect::FrameSummary {
        number,
        layout_ran: false,
        paint_ran: false,
        a11y_ran: false,
        focus_changed: false,
        damage: alacer_core::Damage::NONE,
        viewport: alacer_core::geom::Size::new(120.0, 90.0),
        a11y_updated: 0,
        a11y_removed: 0,
        total_nanos: 0,
        passes: vec![alacer_core::inspect::PassTiming {
            kind: PassKind::Compose,
            ran: false,
            nanos: 0,
        }],
        dirty: Vec::new(),
        dirty_truncated: 0,
    }
}
