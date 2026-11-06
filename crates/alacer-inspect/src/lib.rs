//! Alacer's Causality inspector (§13.3, ADR-0010): the devtools crate that
//! answers *why*, not just *what*.
//!
//! Three parts, each independently testable:
//!
//! - [`recorder::CausalRecorder`] — the [`alacer_core::InspectObserver`]
//!   implementation the pipeline forwards into: bounded ring buffers of
//!   frames, edges, writes, and inputs (every buffer has an eviction policy
//!   and a test that proves it, §14.3), plus the wire-shaped broadcast queue
//!   the server's sessions drain.
//! - [`server::InspectorServer`] — a dependency-light WebSocket server
//!   (RFC 6455 over `std::net`) serving the embedded single-file HTML client
//!   at `GET /` and streaming the documented protocol at `GET /ws`
//!   (`docs/inspector-protocol.md` is normative).
//! - [`wire`] — the versioned JSON vocabulary clients code against; the core
//!   vocabulary (`alacer_core::inspect`) never leaks onto the wire.
//!
//! The intended assembly, from the facade (schematic — the facade owns the
//! real pipeline assembly):
//!
//! ```text
//! let recorder = CausalRecorder::new("demo");
//! pipeline.set_observer(Box::new(recorder.clone_handle()));
//! let server = InspectorServer::bind("127.0.0.1:0", recorder.clone(), "demo")?;
//! let url = server.url();                   // http://127.0.0.1:{port}/
//! std::thread::spawn(move || server.serve_forever());
//! // … run the app; open `url` in a browser.
//! ```
//!
//! Overhead is opt-in (§13.3): a pipeline without an observer records
//! nothing, and every buffer is bounded whatever the app does.

pub mod recorder;
pub mod server;
pub mod wire;

pub use recorder::{CausalRecorder, RecorderHandle};
pub use server::InspectorServer;
pub use wire::{PROTOCOL_VERSION, WireEvent};
