//! The inspector's assembly seam (M83): one call wires the recorder, the
//! observer slot, and the server — the assembly the crate's schematic
//! documented as four hand-spelled steps, now a door with a typed refusal
//! and a reader handle.
//!
//! The door is the schematic's own shape, priced once: the recorder
//! forwards into the shared rings, the server binds before the observer
//! installs (a refused attach changes nothing), and the accept loop runs
//! on its own named thread. The compilable form lives in
//! [`attach_inspector`]'s rustdoc and `crates/alacer/examples/inspector_attach.rs`.

use crate::recorder::CausalRecorder;
use crate::server::InspectorServer;
use alacer_core::passes::PassPipeline;
use std::fmt;

/// Options the attach call takes: the address to serve on and the app name
/// the greeting carries.
///
/// The defaults are the demo canon — the port `inspector_demo` printed and
/// `docs/inspector-protocol.md` documents — with port `0` picking a free
/// one for tests and parallel apps.
#[derive(Debug, Clone)]
pub struct InspectorOptions {
    addr: String,
    app: String,
}

impl InspectorOptions {
    /// The canon options: `127.0.0.1:9217`, the app named "alacer app".
    #[must_use]
    pub fn new() -> Self {
        Self {
            addr: "127.0.0.1:9217".to_string(),
            app: "alacer app".to_string(),
        }
    }

    /// The address the server binds (`host:port`; port `0` lets the OS
    /// pick — the tests' and parallel apps' door).
    #[must_use]
    pub fn addr(mut self, addr: impl Into<String>) -> Self {
        self.addr = addr.into();
        self
    }

    /// The app name the greeting's `app` field carries (the reader's
    /// provenance — which app is this panel watching).
    #[must_use]
    pub fn app(mut self, app: impl Into<String>) -> Self {
        self.app = app.into();
        self
    }
}

impl Default for InspectorOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// The refusal the attach call answers when the server cannot start.
///
/// The address is *data* ([`InspectorAttachError::addr`]), not prose: a
/// consumer resolves it against its own options, and the [`fmt::Display`]
/// still names it for a human. The io cause rides the source chain (the
/// ADR-0023 rule — the failure is data, the `Display` is for a human).
#[derive(Debug)]
pub enum InspectorAttachError {
    /// The listener's bind failed — the port is taken, the address is
    /// unbindable, or the OS refused; the io error is the cause.
    Bind {
        /// The address the bind asked for (the consumer's own fact).
        addr: String,
        /// The io error the bind surfaced.
        source: std::io::Error,
    },
}

impl InspectorAttachError {
    /// The address the refused bind asked for.
    #[must_use]
    pub fn addr(&self) -> &str {
        match self {
            Self::Bind { addr, .. } => addr,
        }
    }
}

impl fmt::Display for InspectorAttachError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bind { addr, source } => {
                write!(f, "the inspector's server could not bind {addr}: {source}")
            }
        }
    }
}

impl std::error::Error for InspectorAttachError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Bind { source, .. } => Some(source),
        }
    }
}

/// The handle the attach call returns: the recorder (a clone — the causal
/// facts, read in-process), the bound port and URL (the panel's door), and
/// the serving thread's join handle.
///
/// Dropping the handle leaves the server serving — a devtools server's
/// lifetime is its app's, and the thread detaches (the accept loop runs
/// until the process exits). [`InspectorHandle::thread`] is the door for
/// the caller that wants to join.
#[derive(Debug)]
pub struct InspectorHandle {
    recorder: CausalRecorder,
    port: u16,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl InspectorHandle {
    /// The recorder: frames, edges, writes, inputs, focus, edits,
    /// announcements — every ring the observer streamed, read in-process
    /// (the tests' and the app's own log window; the rings are the
    /// crate's bounded ones, so a long-lived app never balloons).
    #[must_use]
    pub fn recorder(&self) -> &CausalRecorder {
        &self.recorder
    }

    /// The bound port (`0`-bound attaches answer the OS's pick).
    #[must_use]
    pub fn port(&self) -> u16 {
        self.port
    }

    /// The client URL (`http://127.0.0.1:{port}/`) — the browser's door.
    #[must_use]
    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}/", self.port)
    }

    /// The serving thread, if the spawn succeeded. A `None` here means the
    /// server is bound but unserved — no client can connect — and the
    /// attach's caller owns the decision to spawn it themselves.
    #[must_use]
    pub fn thread(&self) -> Option<&std::thread::JoinHandle<()>> {
        self.thread.as_ref()
    }

    /// Consumes the handle and joins the serving thread. The accept loop
    /// runs until the process exits, so this returns when the thread ends
    /// with it — the explicit-teardown door the tests of a teardown need.
    ///
    /// # Panics
    ///
    /// Panics when the serving thread panicked (a bound listener failing
    /// to accept is unrecoverable for a devtools server — the server's own
    /// contract, surfaced here).
    pub fn join(mut self) {
        if let Some(thread) = self.thread.take()
            && let Err(payload) = thread.join()
        {
            // The documented contract (the `# Panics` clause above): the
            // serving thread's panic is the caller's, not a swallowed fact
            // — the payload rides verbatim, so `panic!` in the accept loop
            // surfaces at the join site with its original message.
            std::panic::panic_any(payload);
        }
    }
}


/// Wires the inspector into `pipeline` in one call: a [`CausalRecorder`]
/// into the observer slot (the causal facts stream, and the trails enable
/// — the slot's own contract), the [`InspectorServer`] bound to
/// `options.addr`, and the accept loop serving on its own thread.
///
/// The order is the failure discipline's: the bind happens *before* the
/// observer installs, so a refused attach (a taken port) leaves the
/// pipeline exactly as it was — no half-attached state, the error the
/// caller reads carrying the address and the io cause.
///
/// # Examples
///
/// ```text
/// // The facade's assembly makes this a three-liner (crates/alacer):
/// let mut pipeline = alacer::pipeline_default();
/// let inspector = alacer::inspect::attach_inspector(
///     &mut pipeline,
///     alacer::inspect::InspectorOptions::new().app("my app"),
/// )
/// .expect("the default port binds");
/// println!("inspector: {}", inspector.url());
/// // … run the app; the recorder streams, the panel watches.
/// ```
///
/// # Errors
///
/// Returns [`InspectorAttachError`] when the server's bind fails.
pub fn attach_inspector(
    pipeline: &mut PassPipeline,
    options: InspectorOptions,
) -> Result<InspectorHandle, InspectorAttachError> {
    // The options move in: the name and the address are the attach's own
    // facts now (the recorder labels itself with the name; the refusal
    // carries the address verbatim).
    let InspectorOptions { addr, app } = options;

    // One recorder, two shapes: the observer handle forwards into the
    // shared rings; the caller's clone reads them in-process (the same
    // `Arc` recording, two doors).
    let recorder = CausalRecorder::new(app.as_str());
    let observer = recorder.clone_handle();
    let reader = recorder.clone();

    // The bind first: a refused attach changes nothing (the failure
    // discipline above).
    let server = InspectorServer::bind(&addr, recorder, app.as_str()).map_err(|source| {
        InspectorAttachError::Bind {
            addr: addr.clone(),
            source,
        }
    })?;
    let port = server.port();

    pipeline.set_observer(Box::new(observer));

    // The accept loop on its own thread — the schematic's last step,
    // named for the reader that debugs a dead panel. A spawn refusal is
    // named too (ADR-0029's observability discipline at the accept
    // loop's own tier): the attach answers, but the trail would have
    // stayed silent with no fact saying why — the warn is the dead
    // panel's one clue, at the same target the session errors ride.
    let thread = match std::thread::Builder::new()
        .name("alacer-inspector".to_string())
        .spawn(move || server.serve_forever())
    {
        Ok(handle) => Some(handle),
        Err(error) => {
            tracing::warn!(
                target: "alacer::inspect",
                %error,
                "the inspector's accept loop failed to spawn; the causal stream stays silent — re-attach to retry"
            );
            None
        }
    };

    tracing::info!(
        target: "alacer::inspect",
        port,
        app = %app,
        "the inspector attached; serving the causal stream"
    );

    Ok(InspectorHandle {
        recorder: reader,
        port,
        thread,
    })
}

#[cfg(test)]
mod tests {
    use super::InspectorHandle;
    use crate::recorder::CausalRecorder;

    #[test]
    fn join_re_raises_the_serving_thread_panic() {
        // The `# Panics` clause is the contract: a serving thread that
        // panicked must not go silently — the joiner panics with the
        // payload the thread carried.
        let thread = std::thread::spawn(|| panic!("accept loop died"));
        let handle = InspectorHandle {
            recorder: CausalRecorder::new("join-test"),
            port: 0,
            thread: Some(thread),
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            handle.join();
        }));
        let payload = result.expect_err("a panicked serving thread must surface at the join");
        // The payload is double-boxed: the thread's panic cargo inside
        // the `Box<dyn Any + Send>` `join` hands back, inside the `Box`
        // `panic_any` raised. Unwrap both, the message verbatim.
        let inner = payload
            .downcast_ref::<Box<dyn std::any::Any + Send>>()
            .and_then(|cargo| {
                cargo
                    .downcast_ref::<&str>()
                    .copied()
                    .or_else(|| cargo.downcast_ref::<String>().map(String::as_str))
            })
            .expect("the payload is the thread's own message");
        assert_eq!(inner, "accept loop died");

        // The clean path: a thread that ended normally joins silently —
        // no spurious panic behind the contract.
        let thread = std::thread::spawn(|| {});
        let handle = InspectorHandle {
            recorder: CausalRecorder::new("join-test"),
            port: 0,
            thread: Some(thread),
        };
        handle.join();
    }
}
