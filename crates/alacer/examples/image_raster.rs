//! The raster tier as a consumer's vocabulary: the picture the app
//! presents is the app's own pixels, riding the scene journal's image
//! primitive end to end.
//!
//! The beats the example exists to show: an embedded PNG decodes into the
//! premultiplied RGBA8 byte contract (`decode_png_rgba`, the tiny-skia
//! codec), the bytes ride the view's `pixels` prop through the registry
//! door the image element already has, and the picture paints (the raster
//! tier — the M138 word's slot stays the borrowed background behind it).
//! The app's wiring moves the raster with one signal: a rebuilt buffer
//! reports the move and repaints, a write that lands where the bytes
//! already are queues nothing (the M21 idle-frame property, at the raster
//! tier), and the words beside the picture never move with the pixels.
//!
//! Run with `cargo run -p alacer --example image_raster`.
//! `ALACER_SOFTWARE=1` forces the softbuffer path (VMs, CI, no-GPU).
//! `--save-screenshot <path>` renders one ticked frame through the
//! software tier and writes it as a PNG — no window, no event loop, no
//! display server (the launch-asset seam: README pixels straight from
//! the full pipeline, image primitive → journal → raster → file). The
//! launch render is the 1024×768 hero shape the README assets want, not
//! the demo window's own 320×240 start size.

use alacer::pipeline_default;
use alacer::platform::{AlacerApp, BackendPreference, PatchChannel};
use alacer_core::FluxRwSignal;
use alacer_core::flux::Flux;
use alacer_core::geom::Size;
use alacer_core::passes::PassPipeline;
use alacer_render::software::decode_png_rgba;
use alacer_view::{Pixels, PropValue, ViewNode, ViewRoot};
use alacer_widgets::default_registry;
use reactive_graph::traits::{GetUntracked as _, Set as _};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

/// The embedded asset: the example's own tiny PNG, decoded at startup
/// into the byte contract the image primitive spells.
const ASSET_PNG: &[u8] = include_bytes!("assets/image_raster.png");

/// The consumer's second picture: the decoded raster with its red and
/// blue channels swapped — the app owns its pixels (the pixel story is
/// the app's own), and a rebuilt buffer is a move under the widget's
/// identity contract, the repaint the swap beat rides.
fn channel_swapped(bytes: &[u8]) -> Arc<[u8]> {
    bytes
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|pixel| [pixel[2], pixel[1], pixel[0], pixel[3]])
        .collect::<Vec<u8>>()
        .into()
}

/// The view: the named picture at its native size (the crisp `Nearest`
/// default), the same picture enlarged (the `sampling` word switching to
/// `Bilinear`), and the words reading back the way the app wired them.
fn image_raster_view(
    original: &Pixels,
    swapped: &Pixels,
    channels_swapped: &FluxRwSignal<bool>,
    wake: &Rc<dyn Fn()>,
) -> ViewNode {
    let shown = if channels_swapped.try_get_untracked().unwrap_or(false) {
        swapped
    } else {
        original
    };
    let status = if channels_swapped.try_get_untracked().unwrap_or(false) {
        "channels: red/blue swapped (a rebuilt buffer — the move repainted)"
    } else {
        "channels: as decoded (the bytes the app embedded)"
    };

    let mut root = ViewNode::element("column")
        .prop("padding", 16.0_f32)
        .prop("gap", 10.0_f32);

    root = root
        .child(ViewNode::element("label").prop("text", "The raster tier (the image primitive)"));

    // The named picture at its native size: the pixels prop rides the
    // registry door the image element already has, the word beside it.
    root = root.child(
        ViewNode::element("image")
            .prop("name", "Band chart")
            .prop("description", "Four color bands as decoded, 24 by 16")
            .prop("width", 24.0_f32)
            .prop("height", 16.0_f32)
            .prop(
                "pixels",
                PropValue::Pixels(Pixels::new(
                    Arc::clone(&shown.bytes),
                    shown.width,
                    shown.height,
                )),
            ),
    );

    // The same picture, enlarged: the box scales the destination, the
    // `sampling` word picks the blended read.
    root = root.child(
        ViewNode::element("image")
            .prop("name", "Band chart, enlarged")
            .prop("description", "The same bands scaled two times, bilinear")
            .prop("width", 48.0_f32)
            .prop("height", 32.0_f32)
            .prop("sampling", "bilinear")
            .prop(
                "pixels",
                PropValue::Pixels(Pixels::new(
                    Arc::clone(&shown.bytes),
                    shown.width,
                    shown.height,
                )),
            ),
    );

    // The app's wiring: one signal, the wake firing the patch channel
    // after the write batch (one wake ⇒ one scoped re-render).
    let next_swapped = *channels_swapped;
    let next_wake = Rc::clone(wake);
    root = root.child(
        ViewNode::element("button")
            .prop("text", "Swap red and blue")
            .prop(
                "on_click",
                PropValue::Handler(Rc::new(RefCell::new(move || {
                    Flux::batch_of(|| next_swapped.set(!next_swapped.get_untracked()));
                    (next_wake.clone())();
                }))),
            ),
    );

    // The words read back the way the app wired them — they never move
    // with the pixels.
    root = root.child(ViewNode::element("label").prop("text", status));

    root
}

/// How this run launches: the windowed app, or the headless screenshot
/// save (`--save-screenshot <path>`).
enum Launch {
    /// The windowed app as always.
    Window,
    /// One ticked frame rendered headless and saved as a PNG.
    Screenshot(PathBuf),
}

/// Parses `--save-screenshot <path>` from `std::env::args` — simple and
/// dependency-free. A bare `--save-screenshot` (no path) refuses honestly
/// with exit code 2: a flag the invocation half-made is a fact about the
/// invocation, never a guess.
fn launch_mode() -> Launch {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(index) = args.iter().position(|arg| arg == "--save-screenshot") else {
        return Launch::Window;
    };
    let Some(path) = args.get(index + 1) else {
        eprintln!("alacer: --save-screenshot needs a path argument");
        std::process::exit(2);
    };
    Launch::Screenshot(PathBuf::from(path))
}

/// The headless save: the mounted view's pipeline ticks exactly once (the
/// 1024×768 launch-asset shape), the frame renders through the software
/// tier — the same stage the softbuffer present path runs — and the PNG
/// lands at `path`. No window, no event loop, no parking: the caller exits
/// right after, cleanly.
fn save_screenshot(pipeline: &mut PassPipeline, path: &Path) {
    let report = pipeline
        .tick(Size::new(1024.0, 768.0))
        .expect("the screenshot frame's layout resolves");
    let mut glyphs = alacer::text::ThreadLocalGlyphSource;
    match alacer::render::screenshot::save_png(
        &report.scene,
        1024,
        768,
        alacer::widgets::theme::SURFACE,
        &mut glyphs,
        path,
    ) {
        Ok(written) => {
            println!(
                "alacer: screenshot saved ({written} bytes) to {}",
                path.display()
            );
        }
        Err(error) => {
            eprintln!("alacer: {error}");
            std::process::exit(1);
        }
    }
}

fn main() {
    // The embedded asset decodes once; a PNG that cannot decode refuses
    // honestly — the example names the asset and stops.
    let (width, height, bytes) = decode_png_rgba(ASSET_PNG)
        .expect("the embedded assets/image_raster.png decodes into premultiplied RGBA8");
    let original = Pixels::new(Arc::clone(&bytes), width, height);
    let swapped = Pixels::new(channel_swapped(&bytes), width, height);

    let mut pipeline = pipeline_default();
    let channel = PatchChannel::new();

    // The app state: one signal, outside the view generation (the ADR-7
    // state-owner contract).
    let channels_swapped = pipeline.flux().signal(false);
    let wake: Rc<dyn Fn()> = Rc::new({
        let channel = channel.clone();
        move || channel.fire()
    });

    let view = image_raster_view(&original, &swapped, &channels_swapped, &wake);
    let mut root = ViewRoot::mount(&view, &mut pipeline, default_registry());

    // The screenshot branch rides out before any window exists: one ticked
    // frame, one file, exit 0 — the CI gate's whole surface.
    if let Launch::Screenshot(path) = launch_mode() {
        save_screenshot(&mut pipeline, &path);
        return;
    }

    let mut app = AlacerApp::new(
        pipeline,
        "Alacer image raster",
        (320.0, 240.0),
        backend_choice(),
    );
    app.set_patch_hook(Box::new(move |pipeline| {
        let view = image_raster_view(&original, &swapped, &channels_swapped, &wake);
        let _report = root.update(&view, pipeline);
    }));
    if let Err(error) = app.with_patch_channel(channel).run() {
        eprintln!("alacer: {error}");
        std::process::exit(1);
    }
}

/// Prefers the software path when `ALACER_SOFTWARE=1` is set.
fn backend_choice() -> BackendPreference {
    if std::env::var_os("ALACER_SOFTWARE").is_some_and(|v| v == "1") {
        BackendPreference::Softbuffer
    } else {
        BackendPreference::Wgpu
    }
}
