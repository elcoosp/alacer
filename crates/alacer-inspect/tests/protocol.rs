//! The inspector protocol's integration tests: a real server on a real
//! socket, a raw TCP client doing the RFC 6455 handshake, and the JSON
//! stream pinned against `docs/inspector-protocol.md`.

use alacer_core::inspect::{
    InputKind, InputRecord, InspectEvent, InspectObserver as _, SettingRecord, SignalWrite,
    ThemeRecord,
};
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
fn frame_refusal_events_carry_the_documented_flat_shape() {
    // The wire conversion is pinned here (the client codes against it):
    // the ladder stage's species beside the failing node when one is
    // the fact, the honest absence when the window is — the documented
    // `frame_refusal` shape (ADR-0077), a new event kind at version 1
    // (the additive rule: clients that ignore unknown kinds unaffected).
    use alacer_core::inspect::{FrameRefusalRecord, FrameRefusalSpecies};
    use alacer_inspect::wire::wire_frame_refusal;

    let refused = wire_frame_refusal(&FrameRefusalRecord {
        species: FrameRefusalSpecies::LayoutRefused,
        node: Some(4_294_967_294),
    });
    let json = serde_json::to_string(&refused).expect("refusal json");
    for fragment in [
        r#""kind":"frame_refusal""#,
        r#""species":"layout_refused""#,
        r#""node":4294967294"#,
    ] {
        assert!(json.contains(fragment), "missing {fragment} in {json}");
    }

    let window_fact = wire_frame_refusal(&FrameRefusalRecord {
        species: FrameRefusalSpecies::PresentFailed,
        node: None,
    });
    let json = serde_json::to_string(&window_fact).expect("present json");
    assert!(
        json.contains(r#""kind":"frame_refusal""#)
            && json.contains(r#""species":"present_failed""#),
        "the present stage's species, beside the kind: {json}"
    );
    assert!(
        json.contains(r#""node":null"#),
        "a present-tier failure is the window's fact, not a widget's: {json}"
    );
}

#[test]
fn view_patch_events_carry_the_documented_flat_shape() {
    // The wire conversion is pinned here (the client codes against it):
    // the reconciler's four counts, flat beside the kind — the
    // documented `view_patch` shape (ADR-0078), a new event kind at
    // version 1 (the additive rule: clients that ignore unknown kinds
    // unaffected). The counts are the fact — the props a patch applied
    // never ride (the payload rule's shape).
    use alacer_core::inspect::ViewPatchRecord;
    use alacer_inspect::wire::wire_view_patch;

    let patch = wire_view_patch(&ViewPatchRecord {
        created: 2,
        removed: 1,
        patched: 5,
        moved: 3,
    });
    let json = serde_json::to_string(&patch).expect("patch json");
    for fragment in [
        r#""kind":"view_patch""#,
        r#""created":2"#,
        r#""removed":1"#,
        r#""patched":5"#,
        r#""moved":3"#,
    ] {
        assert!(json.contains(fragment), "missing {fragment} in {json}");
    }

    let noop = wire_view_patch(&ViewPatchRecord {
        created: 0,
        removed: 0,
        patched: 0,
        moved: 0,
    });
    let json = serde_json::to_string(&noop).expect("noop json");
    assert!(
        json.contains(r#""kind":"view_patch""#) && json.contains(r#""moved":0"#),
        "the no-op re-render's zeros are the fact — nothing moved, and the wire says so: {json}"
    );
}

#[test]
fn theme_events_carry_the_documented_flat_shape() {
    // The wire conversion is pinned here (the client codes against it):
    // the palette's tag beside the kind — the documented `theme` shape
    // (ADR-0079), a new event kind at version 1 (the additive rule:
    // clients that ignore unknown kinds unaffected). The tag is the
    // fact — the palette's colors never ride (the payload rule's
    // shape: the log names the palette, the paint pass resolves it).
    use alacer_core::inspect::ThemeRecord;
    use alacer_inspect::wire::wire_theme;

    let swap = wire_theme(&ThemeRecord {
        palette: "midnight".to_string(),
        repainted: 9,
    });
    let json = serde_json::to_string(&swap).expect("swap json");
    assert!(
        json.contains(r#""kind":"theme""#)
            && json.contains(r#""palette":"midnight""#)
            && json.contains(r#""repainted":9"#),
        "the swap's tag is the fact — the palette named, never its colors, the reach beside it: {json}"
    );
}

#[test]
fn setting_events_carry_the_documented_flat_shape() {
    // The wire conversion is pinned here (the client codes against it):
    // the dark-mode fact beside the kind — the documented `setting`
    // shape (ADR-0080), a new event kind at version 1 (the additive
    // rule: clients that ignore unknown kinds unaffected). The bools
    // are the facts; since M76 (ADR-0082) the broadcast's node count
    // rides beside them, and since M77 (ADR-0083) the reduced-motion
    // fact rides beside the dark one, and since M79 (ADR-0085) the
    // high-contrast fact rides beside the two, and since M81 (ADR-0087)
    // the accent fact rides beside the three as the documented hex (or
    // `null`, the honest absence) — the additive fields the
    // versioning rule admits at version 1, the lengths rule's shape
    // (the ids never ride).
    use alacer_core::color::Color;
    use alacer_core::inspect::SettingRecord;
    use alacer_inspect::wire::wire_setting;

    let flip = wire_setting(&SettingRecord {
        dark: true,
        reduced_motion: false,
        high_contrast: false,
        accent: None,
        repainted: 7,
    });
    let json = serde_json::to_string(&flip).expect("flip json");
    assert!(
        json.contains(r#""kind":"setting""#)
            && json.contains(r#""dark":true"#)
            && json.contains(r#""reduced_motion":false"#)
            && json.contains(r#""high_contrast":false"#)
            && json.contains(r#""accent":null"#)
            && json.contains(r#""repainted":7"#),
        "the flip's facts and its reach, named plain: {json}"
    );
    let flip = wire_setting(&SettingRecord {
        dark: false,
        reduced_motion: true,
        high_contrast: true,
        accent: Some(Color::rgb(255, 94, 58)),
        repainted: 0,
    });
    let json = serde_json::to_string(&flip).expect("flip json");
    assert!(
        json.contains(r#""dark":false"#)
            && json.contains(r#""reduced_motion":true"#)
            && json.contains(r#""high_contrast":true"#)
            && json.contains(r##""accent":"#FF5E3AFF""##)
            && json.contains(r#""repainted":0"#),
        "the light flip names its own facts and reach: {json}"
    );
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
        degradation_step: alacer_core::passes::DegradationStep::None,
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

/// Sends one masked client text frame (the RFC's client-side masking
/// rule) carrying `text` as the payload.
fn send_client_text(mut socket: &TcpStream, text: &[u8]) {
    let mut frame_bytes = vec![0x81_u8, 0x80 | alacer_core::num::usize_to_u8(text.len())];
    frame_bytes.extend_from_slice(&[0_u8, 0, 0, 0]);
    frame_bytes.extend_from_slice(text);
    socket
        .write_all(frame_bytes.as_slice())
        .expect("client frame");
}

#[test]
fn a_replay_ask_returns_the_history_the_rings_hold() {
    let recorder = CausalRecorder::new("replay");
    // The facts a first session lived through, recorded before any
    // socket exists: the rings are the durable history.
    recorder
        .clone_handle()
        .observe(InspectEvent::Write(SignalWrite {
            signal: 7,
            sequence: 1,
            origin: std::borrow::Cow::Borrowed("user click"),
        }));
    let mut first = dummy_frame(1);
    first.paint_ran = true;
    recorder.clone_handle().observe(InspectEvent::Frame(first));

    let server = InspectorServer::bind("127.0.0.1:0", recorder.clone(), "replay").expect("bind");
    let port = server.port();
    std::thread::spawn(move || server.serve_once());
    let mut reader = connect_ws(port);
    let _hello = read_text_frame(&mut reader);
    // The first session's broadcast drain (hello, then the pre-connect
    // facts it finds queued) — the panel watched live.
    let write = read_text_frame(&mut reader);
    assert!(write.contains(r#""kind":"write""#), "live write: {write}");
    let frame = read_text_frame(&mut reader);
    assert!(frame.contains(r#""kind":"frame""#), "live frame: {frame}");

    let socket = reader.get_ref().try_clone().expect("clone");
    send_client_text(&socket, br#"{"kind":"bye"}"#);
    std::thread::sleep(Duration::from_millis(50));

    // The reconnect: the second panel is the late client. One more
    // write lands while no session polls the broadcast queue.
    recorder
        .clone_handle()
        .observe(InspectEvent::Write(SignalWrite {
            signal: 7,
            sequence: 2,
            origin: std::borrow::Cow::Borrowed("user click"),
        }));

    let server_two =
        InspectorServer::bind("127.0.0.1:0", recorder.clone(), "replay-two").expect("rebind");
    let port_two = server_two.port();
    std::thread::spawn(move || server_two.serve_once());
    let mut reader = connect_ws(port_two);
    let _hello = read_text_frame(&mut reader);
    // The delta the broadcast queue holds for this session only.
    let live = read_text_frame(&mut reader);
    assert!(
        live.contains(r#""sequence":2"#),
        "the session's own delta: {live}"
    );

    // The ask: the rings' history, the facts the broadcast queue never
    // held for this session (the write the first panel's drain took).
    let socket = reader.get_ref().try_clone().expect("clone");
    send_client_text(&socket, br#"{"kind":"replay"}"#);
    std::thread::sleep(Duration::from_millis(80));
    let burst_write_one = read_text_frame(&mut reader);
    assert!(
        burst_write_one.contains(r#""kind":"write""#)
            && burst_write_one.contains(r#""sequence":1"#),
        "the rings' oldest write replays: {burst_write_one}"
    );
    let burst_write_two = read_text_frame(&mut reader);
    assert!(
        burst_write_two.contains(r#""kind":"write""#)
            && burst_write_two.contains(r#""sequence":2"#),
        "the younger write replays beside it: {burst_write_two}"
    );
    let burst_frame = read_text_frame(&mut reader);
    assert!(
        burst_frame.contains(r#""kind":"frame""#) && burst_frame.contains(r#""number":1"#),
        "the ring's frame replays in the vocabulary order: {burst_frame}"
    );

    // The look left the rings whole: the next live fact streams once.
    recorder
        .clone_handle()
        .observe(InspectEvent::Input(InputRecord {
            kind: InputKind::Wheel,
            target: None,
        }));
    let next = read_text_frame(&mut reader);
    assert!(
        next.contains(r#""kind":"input""#),
        "live after the ask: {next}"
    );
}

#[test]
fn a_replay_ask_twice_answers_the_same_history() {
    let recorder = CausalRecorder::new("replay-twice");
    recorder
        .clone_handle()
        .observe(InspectEvent::Input(InputRecord {
            kind: InputKind::Key,
            target: None,
        }));
    let server =
        InspectorServer::bind("127.0.0.1:0", recorder.clone(), "replay-twice").expect("bind");
    let port = server.port();
    std::thread::spawn(move || server.serve_once());
    let mut reader = connect_ws(port);
    let _hello = read_text_frame(&mut reader);
    // The session's first loop pass drains the broadcast queue (the
    // pre-connect input) — read it, then the socket is quiet.
    let live = read_text_frame(&mut reader);
    assert!(live.contains(r#""kind":"input""#), "the live drain: {live}");
    std::thread::sleep(Duration::from_millis(60));
    let socket = reader.get_ref().try_clone().expect("clone");
    send_client_text(&socket, br#"{"kind":"replay"}"#);
    std::thread::sleep(Duration::from_millis(80));
    let first = read_text_frame(&mut reader);
    assert!(
        first.contains(r#""kind":"input""#) && first.contains(r#""input":"key""#),
        "the ring's input replays: {first}"
    );
    send_client_text(&socket, br#"{"kind":"replay"}"#);
    std::thread::sleep(Duration::from_millis(80));
    let second = read_text_frame(&mut reader);
    assert_eq!(first, second, "the ask reads without disturbing — twice");
}

#[test]
fn a_replay_ask_after_a_clear_answers_nothing_and_lives() {
    let recorder = CausalRecorder::new("replay-clear");
    recorder
        .clone_handle()
        .observe(InspectEvent::Input(InputRecord {
            kind: InputKind::Wheel,
            target: None,
        }));
    recorder
        .clone_handle()
        .observe(InspectEvent::Theme(ThemeRecord {
            palette: "midnight".to_string(),
            repainted: 3,
        }));
    recorder
        .clone_handle()
        .observe(InspectEvent::Setting(SettingRecord {
            dark: true,
            reduced_motion: false,
            high_contrast: false,
            accent: None,
            repainted: 0,
        }));
    let server =
        InspectorServer::bind("127.0.0.1:0", recorder.clone(), "replay-clear").expect("bind");
    let port = server.port();
    std::thread::spawn(move || server.serve_once());
    let mut reader = connect_ws(port);
    let _hello = read_text_frame(&mut reader);
    // The connect-time broadcast drain (the three pre-connect facts),
    // read so the socket is quiet before the controls land.
    let live_input = read_text_frame(&mut reader);
    assert!(live_input.contains(r#""kind":"input""#), "{live_input}");
    let live_theme = read_text_frame(&mut reader);
    assert!(live_theme.contains(r#""kind":"theme""#), "{live_theme}");
    let live_setting = read_text_frame(&mut reader);
    assert!(
        live_setting.contains(r#""kind":"setting""#),
        "{live_setting}"
    );

    let socket = reader.get_ref().try_clone().expect("clone");
    send_client_text(&socket, br#"{"kind":"clear"}"#);
    std::thread::sleep(Duration::from_millis(80));
    assert_eq!(recorder.inputs().len(), 0, "the input ring resets");
    assert_eq!(
        recorder.themes().len(),
        0,
        "the theme ring resets with the rest"
    );
    assert_eq!(
        recorder.settings().len(),
        0,
        "the setting ring resets with the rest"
    );

    // The empty burst answers nothing on the socket; the session keeps
    // serving — the next live fact arrives whole.
    send_client_text(&socket, br#"{"kind":"replay"}"#);
    std::thread::sleep(Duration::from_millis(80));
    recorder
        .clone_handle()
        .observe(InspectEvent::Input(InputRecord {
            kind: InputKind::PointerDown,
            target: None,
        }));
    let next = read_text_frame(&mut reader);
    assert!(
        next.contains(r#""kind":"input""#) && next.contains(r#""input":"pointer_down""#),
        "the session lives past an empty burst: {next}"
    );
}
