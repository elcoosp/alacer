//! Alacer's rendering: the scene journal and the software rasterizer (A8: the
//! software path is first-class).
//!
//! - [`SceneJournalImpl`]: per-node layers keyed by node id; the paint pass
//!   re-emits only dirty layers, clean layers replay as recorded (§9.1).
//! - [`SoftwareRenderer`]: rasterizes a
//!   [`SceneSnapshot`](alacer_core::scene::SceneSnapshot) through tiny-skia —
//!   the headless/CI/low-end backend (ADR-2), and the rig every test runs on.
//! - [`screenshot`]: the headless save path — one rendered frame → one PNG
//!   file, no window and no display server (the launch-asset seam).

pub mod journal;
pub mod screenshot;
pub mod software;

pub use journal::{SceneBatches, SceneJournalImpl, partition};
pub use screenshot::ScreenshotError;
pub use software::{ClosureGlyphSource, SoftwareRenderer};
