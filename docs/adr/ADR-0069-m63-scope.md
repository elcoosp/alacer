# ADR-0069: M63 scope — the trail's provided reader joins the seam

- **Status:** Accepted (2026-11-14)
- **Implements:** the collector ask the m-suite pre-registered — every
  gate file and every trail example hand-rolled the same shape (the
  `Rc<RefCell<VecDeque>>` behind a one-method observer, the m-suite's
  "helper shape" repeated 33 files over; the benches' `Share` wrapper
  beside it), and the harness could not install a reader without each
  test writing its own — a consumer prices the *collector* → the seam
  grows the reader the way the clipboard seam grew its own provided
  implementation, never a per-file collector the next suite copies
  again
- **Companion:** ADR-0005 (testing productization — the harness is the
  product's own validation surface), ADR-0010 (the additive wire rule's
  in-process twin: the reader holds the events the routes already
  stream, no new kind), ADR-0020 (the clipboard seam's provided
  `MemoryClipboard` — the precedent this follows), ADR-0011 (the
  secret-safety rule at every tier: the reader holds the records' own
  shapes — lengths and species — never a payload), A7 (the idle
  contract: the trail-off pipeline streams nothing, and a trail nobody
  drains is bounded), architecture §13.3, §18 (the M63 roadmap row)

## Decision

- **The reader is provided beside the trait.**
  [`TrailLog`](crates/alacer-core/src/trail.rs) lives in `alacer-core`
  beside [`InspectObserver`](crates/alacer-core/src/inspect.rs) — the
  seam's own implementation, clonable, `Arc<Mutex>`-backed so one
  installation serves many readers. The observing half is the trait's
  one method; the reading half is the window's: `drain` hands the held
  events over oldest-first and empties the window, `look` reads without
  disturbing it, `last` names the tip, `dropped` names what the ring
  evicted — the eviction counted, never a silent loss (the
  named-refusal rule at the reader's own tier).
- **The window is bounded and priced.** `TRAIL_RING_CAP` (1,024) is the
  default window; `with_cap` prices a reader's own. A ring nobody
  drains evicts its oldest and counts the eviction — a devtools log,
  never a frame-budget liability. A `0` cap is refused with the house's
  honest panic (a reader that remembers nothing would silently drop
  every fact).
- **The harness grows the one-call door.** `TestHarness::trail` (and
  `trail_with_cap`) installs the reader on the pipeline and hands back
  the reading handle — one line where every suite wrote eleven. The
  `pipeline_mut().set_observer` escape hatch stays for the suites that
  want their own observer; the door is the provided path.
- **The corpus grows by the coordinated rule (ADR-0011).** The reader's
  bench prices the route it serves (`trail/reader_announce_100_1k`, the
  announce shape with the reader installed + drained;
  `trail_ring/fill_drain_1024`, the reader's own lifecycle) — gated by
  the `trail_reader_overhead` budget (≤ 2× the unobserved
  `announce/route_100_announcements_1k`: the trail rides the observe
  branch an installed observer already pays, never a re-price of the
  route) and two same-host regression gates; the m5/m6 corpus pins
  re-agree to 68. `examples/trail_reader.rs` is the consumer
  demonstration: the app reads its own log through the provided reader,
  no hand-rolled collector in sight.
- **No wire change.** The reader holds the events the routes already
  stream — no new protocol kind, no version bump, the inspector's wire
  untouched.
