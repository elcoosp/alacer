//! The inspector's HTTP-client proof, headless (§13.3): `GET /` serves the
//! embedded single-file HTML client out of the box — a real bind on an
//! OS-picked port, a real request, and the exact bytes a browser renders.
//! The protocol doc's claim ("The embedded single-file HTML client (the
//! standalone inspector app)") is asserted byte-for-byte against the
//! `include_str!`-embedded source, so a stub or a missing file fails here
//! before a user ever opens the URL.

use alacer_inspect::server::inspector_html;
use alacer_inspect::{CausalRecorder, InspectorServer};
use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::net::TcpStream;

/// One `GET /` round trip against a freshly bound server: the response
/// head (status line + headers, the blank line dropped) and the body. The
/// server answers `Connection: close`, so the body read runs to EOF.
fn fetch_root(port: u16) -> (Vec<String>, String) {
    let mut socket = TcpStream::connect(("127.0.0.1", port)).expect("connect to the bound server");
    socket
        .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .expect("the request head");
    let mut reader = BufReader::new(socket);
    let mut head = Vec::new();
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).expect("a response head line");
        let trimmed = line.trim_end().to_string();
        let head_done = trimmed.is_empty();
        head.push(trimmed);
        if head_done {
            break;
        }
    }
    let mut body = String::new();
    reader.read_to_string(&mut body).expect("the response body");
    (head, body)
}

/// Binds a fresh inspector server on an OS-picked port and spawns its
/// one-connection accept (the test seam): the address lives, the request
/// has a listener to land on.
fn bound_server(app: &str) -> u16 {
    let recorder = CausalRecorder::new(app);
    let server = InspectorServer::bind("127.0.0.1:0", recorder, app).expect("bind 127.0.0.1:0");
    let port = server.port();
    std::thread::spawn(move || server.serve_once());
    port
}

#[test]
fn get_root_serves_the_embedded_html_client_out_of_the_box() {
    let port = bound_server("html-client");
    let (head, body) = fetch_root(port);

    assert!(
        head[0].starts_with("HTTP/1.1 200 OK"),
        "the client ships at 200 OK, got {:?}",
        head[0]
    );
    let content_type = head
        .iter()
        .find(|line| line.to_ascii_lowercase().starts_with("content-type:"))
        .expect("a Content-Type header on the client response");
    assert!(
        content_type.to_ascii_lowercase().contains("text/html"),
        "the client is HTML, got {content_type:?}"
    );
    let connection = head
        .iter()
        .find(|line| line.to_ascii_lowercase().starts_with("connection:"))
        .expect("a Connection header (the one-request close contract)");
    assert!(
        connection.to_ascii_lowercase().contains("close"),
        "the client response closes the socket, got {connection:?}"
    );
    assert!(
        body.starts_with("<!doctype html>"),
        "the body opens the HTML document, got {body:.40}"
    );
    assert!(body.contains("<html"), "the <html> element rides");
    assert!(
        body.contains("<title>") && body.contains("Causality Inspector"),
        "the standalone app's title rides"
    );
    assert!(
        body.contains("new WebSocket("),
        "the WebSocket connection script rides — the client speaks /ws"
    );
}

#[test]
fn the_served_bytes_are_the_embedded_single_file_client() {
    let port = bound_server("html-client-identity");
    let (head, body) = fetch_root(port);

    assert!(
        head[0].starts_with("HTTP/1.1 200 OK"),
        "the identity probe needs the served document, got {:?}",
        head[0]
    );
    assert_eq!(
        body,
        inspector_html(),
        "GET / serves the include_str!-embedded client, byte for byte"
    );
    let content_length = head
        .iter()
        .find(|line| line.to_ascii_lowercase().starts_with("content-length:"))
        .expect("a Content-Length header on the client response");
    let declared = content_length
        .split_once(':')
        .map(|(_, value)| value.trim().parse::<u64>())
        .expect("a numeric Content-Length")
        .expect("a parseable Content-Length");
    assert_eq!(
        declared,
        u64::try_from(body.len()).unwrap_or(u64::MAX),
        "the declared length is the body's byte count"
    );
}
