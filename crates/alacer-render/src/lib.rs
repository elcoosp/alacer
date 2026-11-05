//! Alacer's rendering: the scene journal and the software rasterizer (A8: the
//! software path is first-class).
//!
//! - [`SceneJournalImpl`]: per-node layers keyed by node id; the paint pass
//!   re-emits only dirty layers, clean layers replay as recorded (§9.1).
//! - [`SoftwareRenderer`]: rasterizes a
//!   [`SceneSnapshot`](alacer_core::scene::SceneSnapshot) through tiny-skia —
//!   the headless/CI/low-end backend (ADR-2), and the rig every test runs on.

pub mod journal;
pub mod software;

pub use journal::{SceneBatches, SceneJournalImpl, partition};
pub use software::{ClosureGlyphSource, SoftwareRenderer};
