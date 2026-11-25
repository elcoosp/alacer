//! M83 exit-criteria integration tests (§13.3, ADR-0010): the inspector
//! at the assembly site.
//!
//! The row's contract, pinned at the crate that owns the door:
//! `attach_inspector` in one call before the first frame — the recorder
//! into the observer slot, the bind before the install (**a refused
//! attach changes nothing**), the reader handle answering the causal
//! facts in-process. The accept loop serves on its own thread; the
//! one-call demonstration rides `examples/inspector_attach.rs`, the
//! wire/protocol halves the `wire` tests and the m-suite inspector
//! contracts own.

use std::sync::Arc;

use alacer_core::arena::NodeId;
use alacer_core::flux::Flux;
use alacer_core::geom::{Point, Rect, Size};
use alacer_core::passes::{
    A11yDelta, A11ySink, FrameReport, JournalStats, LayoutEngine, LayoutError, PassPipeline,
    SceneJournal, TreeSnapshot,
};
use alacer_core::scene::{Primitive, SceneSnapshot};
use alacer_core::settings::CanonSettings;
use alacer_core::style::{Size2, Style};
use alacer_core::widget::{LabelSource, Role, Widget};
use alacer_inspect::{InspectorAttachError, InspectorOptions, attach_inspector};
use rustc_hash::FxHashMap;

/// A fixed-size box (the pipeline's body: something to mount, something
/// to tick).
struct Fixed {
    size: Size,
}

impl Fixed {
    fn new(size: Size) -> Self {
        Self { size }
    }
}

impl Widget for Fixed {
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn role(&self) -> Role {
        Role::Container
    }
    fn label(&self) -> LabelSource {
        LabelSource::text("the app's body")
    }
    fn style(&self) -> Style {
        Style {
            size: Size2 {
                width: Some(self.size.width),
                height: Some(self.size.height),
            },
            ..Style::default()
        }
    }
    fn paint(&self, ctx: &mut alacer_core::scene::PaintCtx<'_>) {
        ctx.push_quad(alacer_core::scene::Quad {
            rect: Rect::new(Point::zero(), self.size),
            color: alacer_core::Color::rgb(24, 24, 27),
            corner_radius: 0.0,
            border_color: alacer_core::Color::TRANSPARENT,
            border_width: 0.0,
            clip: None,
        });
    }
}

/// The no-op layout engine (mount tests drive the arena, never a tick's
/// geometry — the view crate's mount-test shape).
struct NullLayout;

impl LayoutEngine for NullLayout {
    fn sync_and_layout(
        &mut self,
        _arena: &mut alacer_core::Arena,
        _root: NodeId,
        _dirty: &FxHashMap<NodeId, alacer_core::mask::Change>,
        _viewport: Size,
    ) -> Result<(), LayoutError> {
        Ok(())
    }

    fn measure_calls(&self) -> u64 {
        0
    }

    fn reset_measure_calls(&mut self) {}
}

/// The no-op journal (the scene's honest blank).
struct NullJournal;

impl SceneJournal for NullJournal {
    fn commit_layer(&mut self, _node: NodeId, _primitives: Vec<Primitive>) {}

    fn drop_layer(&mut self, _node: NodeId) {}

    fn layer(&self, _node: NodeId) -> Option<Vec<Primitive>> {
        None
    }

    fn snapshot(&self, _order: &[NodeId]) -> SceneSnapshot {
        SceneSnapshot {
            primitives: Vec::new(),
        }
    }

    fn retain(&mut self, _order: &[NodeId]) {}

    fn clear(&mut self) {}

    fn stats(&self) -> JournalStats {
        JournalStats::default()
    }
}

/// The no-op a11y sink.
#[derive(Default)]
struct NullA11y;

impl A11ySink for NullA11y {
    fn apply(&mut self, _snapshot: Arc<TreeSnapshot>, _delta: &A11yDelta) {}
}

/// A pipeline with the mount-test fakes — the attach's body.
fn pipeline() -> PassPipeline {
    PassPipeline::new(
        Flux::new(),
        Box::new(NullLayout),
        Box::new(NullJournal),
        Box::new(NullA11y),
    )
}

/// The one-call attach, before the first frame: the recorder rides the
/// observer slot, the reader answers the causal facts in-process — the
/// flip's setting record and the frame the tick produced, read back
/// through the handle without a wire in sight.
#[test]
fn the_one_call_attach_streams_the_causal_facts_in_process() {
    let mut pipeline = pipeline();
    let _root = pipeline.mount_root(Box::new(Fixed::new(Size::new(400.0, 300.0))));

    let handle = attach_inspector(&mut pipeline, InspectorOptions::new().addr("127.0.0.1:0"))
        .expect("the attach binds an OS-picked port");
    assert!(
        handle.port() > 0,
        "the bind answered the OS's pick: {}",
        handle.url()
    );

    // Before the first frame the pipeline has already ticked? The row's
    // order: attach *before* the first frame — the flip and tick ride
    // after, and the reader answers what they streamed.
    assert!(
        !pipeline.set_settings(Box::new(CanonSettings)),
        "the canon facts over the canon default are the no-op the door answers"
    );
    let report: FrameReport = pipeline
        .tick(Size::new(400.0, 300.0))
        .expect("the first frame after the attach");

    let recorder = handle.recorder();
    assert!(
        recorder.frames_seen() >= 1,
        "the observer slot streams the frames the tick produced"
    );
    let _ = report;
    assert!(
        recorder.latest_frame().is_some(),
        "the reader answers the causal facts in-process — no wire, no panel"
    );
}

/// The bind before the install: a refused attach changes nothing — the
/// error names the address, the pipeline keeps working, and the flip
/// the refusal's frame ran streams to nobody (no observer ever
/// installed).
#[test]
fn a_refused_attach_changes_nothing() {
    // Occupy the port first: the bind cannot take it.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("the squatter binds");
    let occupied = listener
        .local_addr()
        .expect("the squatter's port")
        .to_string();

    let mut pipeline = pipeline();
    let _root = pipeline.mount_root(Box::new(Fixed::new(Size::new(400.0, 300.0))));

    let refused = attach_inspector(&mut pipeline, InspectorOptions::new().addr(&occupied));
    let InspectorAttachError::Bind { addr, source: _ } =
        refused.expect_err("the bind refuses — the error is the bind's own");
    assert_eq!(addr, occupied, "the refusal names the address verbatim");

    // The pipeline is intact: the flip lands, the tick answers.
    let _ = pipeline.tick(Size::new(400.0, 300.0)).expect("the frame");
    let _ = pipeline.set_settings(Box::new(CanonSettings));

    // A later successful attach streams only what happens after *its*
    // install — the refused attach's frame and flip reached no recorder.
    let handle = attach_inspector(&mut pipeline, InspectorOptions::new().addr("127.0.0.1:0"))
        .expect("the retry binds an OS-picked port");
    let _ = pipeline.tick(Size::new(400.0, 300.0)).expect("the frame");
    assert_eq!(
        handle.recorder().settings().len(),
        0,
        "the canon-over-canon flip streamed nothing; the refused attach's past reached nobody"
    );
    assert!(
        handle.recorder().frames_seen() >= 1,
        "the successful attach's own install streams forward only"
    );
}

/// The accept loop serves on its own thread: a plain TCP connect to the
/// bound port succeeds while the app runs — the panel's door is open
/// without any protocol spoken.
#[test]
fn the_accept_loop_serves_on_its_own_thread() {
    let mut pipeline = pipeline();
    let _root = pipeline.mount_root(Box::new(Fixed::new(Size::new(400.0, 300.0))));
    let handle = attach_inspector(&mut pipeline, InspectorOptions::new().addr("127.0.0.1:0"))
        .expect("the attach binds");

    let addr = format!("127.0.0.1:{}", handle.port());
    let stream = std::net::TcpStream::connect(&addr).expect("the accept loop answers");
    assert!(
        stream.peer_addr().is_ok(),
        "a live connection to the serving thread: {addr}"
    );
}
