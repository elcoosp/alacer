# Changelog

All notable changes to **Alacer** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
generated from Conventional Commits by [git-cliff](https://git-cliff.org).

## [0.3.0] — 2026-11-05



### Added


- Add the Change mask invalidation currency

- Add the single cast seam

- Add shared euclid-backed geometry types

- Add the RGBA color type

- Add id-path prefix hashing

- Add the pointer event vocabulary

- Add typed layout styles and the measure seam

- Add the Widget trait with required role and label

- Add the retained widget arena

- Add the flux bridge over reactive_graph

- Add the scene primitive vocabulary and PaintCtx

- Add the pass contract and PassPipeline

- Wire the core crate public surface

- Add keyed reconciliation

- Add the Alacer-to-Taffy style conversion

- Add the LRU measure cache

- Add torin-style dirty-reason semantics

- Add the persistent Taffy LayoutHost

- Wire the layout crate public surface

- Add the frame-windowed layout cache

- Add the font registry

- Add the zeno glyph rasterizer

- Add the parley facade and thread-local context

- Add the node-layered scene journal

- Add the tiny-skia software rasterizer

- Wire the render crate public surface

- Add the AccessKit sink with incremental TreeUpdates

- Add the Panel decoration

- Add Column, Row and Spacer containers

- Add the Label widget

- Add the Button widget

- Add the scrolling List

- Wire the widget family, theme and BoxedWidget

- Add the event coalescer

- Add the softbuffer present path

- Add the wgpu renderer

- Add the AlacerApp winit driver

- Add the headless TestHarness

- Add the alacer facade crate

- Add u32_to_f32, u64_to_f32 and u128_to_u64 helpers

- Add the EventPhase capture/bubble enum

- Add Widget::focusable and the as_any_mut escape hatch

- Add the FluxRwSignal type alias

- Add the text seam (widgets shape text without alacer-text)

- Add reorder_children, is_live, and drain reporting

- Implement the ten-pass contract with the overhead gate

- Wire text_seam and re-export the M1 pass vocabulary

- Add the ThreadLocalTextSeam adapter

- Add the ViewNode tree and PropValue vocabulary

- Add the WidgetRegistry element table

- Add the ViewRoot in-place reconciler

- Wire the view tree, registry and reconciler modules

- Add the rsx! view DSL proc-macro

- Add in-place setters, as_any_mut and the text seam

- Add style_mut, as_any_mut and Spacer::set_size

- Add in-place setters, as_any_mut and the text seam

- Add style_mut and as_any_mut

- Add set_height, set_color and as_any_mut

- Register the default M0 element set

- Expose the default element registry

- Install the text seam at AlacerApp assembly

- Add the a11y query builder, press, and the text seam

- Expose rsx! and mount_view through the facade

- The root candidate, the bounded publish, and the sync diff

- The cached scene replay


### Benchmarks


- Add arena build/mask/walk benchmarks

- Add flux batch-coalescing and drain benchmarks

- Add reconciliation benchmarks

- Add the incremental vs full relayout benchmark

- Add cold/cached text layout benchmarks

- Add journal commit and replay benchmarks

- Add the §7.2 pass-overhead churn benchmark


### Build


- Scaffold the Alacer workspace

- Pin the initial dependency graph

- Cargo-deny license gate (no AGPL, no unknown licenses)

- Declare the alacer-core crate

- Declare the alacer-view crate

- Declare the alacer-layout crate

- Declare the alacer-text crate

- Declare the alacer-render crate

- Declare the alacer-a11y crate

- Declare the alacer-widgets crate

- Declare the alacer-platform crate

- Declare the alacer-testing crate

- Declare the alacer facade crate

- Register the pass_overhead benchmark target

- Declare the alacer-macro proc-macro crate

- Tighten to core + view (the M1 DAG close-out)

- Add alacer-macro and alacer-view deps

- Add alacer-macro to the facade s dependencies


### Continuous Integration


- Add the crate layering DAG checker

- Add the fmt/clippy/test/doc/deny/dag pipeline

- Add the git-cliff release automation

- Extend the layering for alacer-macro and the M1 tightening

- Ship the counter_view example binary


### Documentation


- Add the counter example

- Add the scrolling list example

- Add the git-cliff configuration

- Add the initial CHANGELOG

- Add the CONTRIBUTING guide

- Add the system architecture document

- Record the M0 scope decision

- Record the workspace discipline decision

- Record the scheduling contract decision

- Record the persistent-Taffy decision

- Record the software-first glyph-seam decision

- Record the testing productization decision

- Record the release automation decision

- Add the counter_view rsx! example

- Record the M1 scope decision

- Add the 0.2.0 release notes

- Update README for the M1 milestone

- Record the M2 scope decision

- The M2 milestone record and the 187-test count


### Fixed


- Treat dead arena slots as not-live during taffy sync

- Repair the broken snapshot corpus — glyph blend, y-convention, stretch, compose


### Miscellaneous


- Ignore build artifacts and editor/OS debris

- Pin the 1.99.1 toolchain with clippy and rustfmt

- Set rustfmt max_width to 100

- Add the Apache-2.0 license text

- Add the MIT license text

- Regenerate the lockfile for the 0.2.0 workspace

- Bless broken pngs

- Readme

- Logo & deny


### Tests


- Add the counter snapshot golden

- Add the scrolled-list snapshot golden

- Add the M0 exit-criteria integration tests

- Implement as_any_mut on the arena bench Node

- Implement as_any_mut on the incremental test Probe

- Implement as_any_mut on the incremental bench TestBox

- Add the M1 exit criteria and 30-scenario corpus

- Add the rsx! end-to-end integration suite

<!-- generated by git-cliff -->
