//! The inspector's assembly seam (M83): one `attach_inspector` call wires
//! the causal recorder into the observer slot and serves the devtools
//! protocol on a real socket — the wiring `inspector_demo` used to spell
//! by hand, now a facade door with a typed refusal and a reader handle.

use alacer::inspect::InspectorOptions;
use alacer_core::announce::Politeness;
use alacer_testing::TestHarness;
use std::io::{BufRead as _, BufReader, Write};
use std::net::TcpStream;

#[test]
fn attach_binds_serves_and_streams_causal_facts() {
    let mut harness = TestHarness::new(320.0, 240.0);

    let handle = alacer::inspect::attach_inspector(
        harness.pipeline_mut(),
        InspectorOptions::new()
            .app("the attach test")
            .addr("127.0.0.1:0"),
    )
    .expect("port 0 always binds");

    // The bound facts: a real port and the URL a panel opens.
    assert!(handle.port() > 0, "the OS assigned a port");
    assert_eq!(handle.url(), format!("http://127.0.0.1:{}/", handle.port()));
    assert_eq!(handle.recorder().app(), "the attach test");

    // The observer rides the pipeline: one tick, one streamed frame —
    // the reader's honest count (never evicted).
    harness.tick();
    assert_eq!(
        handle.recorder().frames().len(),
        1,
        "the recorder carried the tick's frame"
    );
    assert_eq!(handle.recorder().dropped(), 0, "no fact dropped");

    // The serve thread is live: a plain HTTP client gets the panel's HTML.
    let mut stream =
        TcpStream::connect(("127.0.0.1", handle.port())).expect("the serve thread accepts");
    stream
        .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .expect("the request writes");
    let mut reader = BufReader::new(stream);
    let mut status = String::new();
    reader.read_line(&mut status).expect("the status line");
    assert!(
        status.starts_with("HTTP/1.1 200"),
        "the panel serves: {status}"
    );
    let mut head = String::new();
    let mut bytes = 0_usize;
    while bytes < 4096 {
        let mut line = String::new();
        let read = reader.read_line(&mut line).expect("the head reads");
        if read == 0 || line == "\r\n" {
            break;
        }
        bytes += read;
        head.push_str(&line);
    }
    assert!(
        head.to_ascii_lowercase().contains("text/html"),
        "the panel's content type rides the head: {head}"
    );
}

#[test]
fn attach_refuses_a_taken_port_with_the_source_chain() {
    // Occupy a port, then aim the attach at it: the typed refusal names
    // the address and chains the io cause (the ADR-0023 rule — the failure
    // is data, the Display is for a human).
    let squat = std::net::TcpListener::bind("127.0.0.1:0").expect("the squatter binds");
    let port = squat.local_addr().expect("the squatter's port").port();

    let mut harness = TestHarness::new(320.0, 240.0);
    let error = alacer::inspect::attach_inspector(
        harness.pipeline_mut(),
        InspectorOptions::new()
            .app("the refusal test")
            .addr(format!("127.0.0.1:{port}")),
    )
    .expect_err("the taken port refuses");

    assert!(
        matches!(error, alacer::inspect::InspectorAttachError::Bind { .. }),
        "the refusal is the bind's"
    );
    let displayed = error.to_string();
    assert!(
        displayed.contains(&port.to_string()),
        "the refusal names the port: {displayed}"
    );
    let source = std::error::Error::source(&error);
    assert!(source.is_some(), "the io cause rides the chain");

    // The failed attach changed nothing: the pipeline still ticks.
    harness.tick();
}

#[test]
fn attach_streams_the_drained_trails_the_observer_enables() {
    // set_observer enables the trails; the recorder's rings carry their
    // facts. One announcement lands in the announce ring after a tick.
    let mut harness = TestHarness::new(320.0, 240.0);
    let handle = alacer::inspect::attach_inspector(
        harness.pipeline_mut(),
        InspectorOptions::new()
            .app("the trail test")
            .addr("127.0.0.1:0"),
    )
    .expect("port 0 always binds");

    alacer::core::announce::announce("The attach demo speaks", Politeness::Polite);
    harness.tick();

    assert!(
        !handle.recorder().announces().is_empty(),
        "the announcement trail streamed through the attached recorder"
    );
}
